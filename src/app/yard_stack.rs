use std::sync::Arc;
use std::thread;

use crate::{AfterFlow, ArcYard, Link, Flow, story, yard};
use crate::yard::{overlay, YardObservable};

pub(crate) struct YardStack;

#[derive(Clone)]
pub(crate) struct State {
	era: usize,
	yard: ArcYard,
	back_to_front: Vec<Arc<dyn YardObservable>>,
	report_link: Option<Link<()>>,
}

impl State {
	pub fn pop_front(&self) -> Self {
		let mut back_to_front = self.back_to_front.to_vec();
		back_to_front.pop();
		State {
			era: self.era + 1,
			yard: self.yard.to_owned(),
			back_to_front,
			report_link: self.report_link.to_owned(),
		}
	}
	pub fn push_front(&self, front: Arc<dyn YardObservable>) -> Self {
		let mut back_to_front = self.back_to_front.to_vec();
		back_to_front.push(front);
		State {
			era: self.era + 1,
			yard: self.yard.to_owned(),
			back_to_front,
			report_link: self.report_link.to_owned(),
		}
	}
	pub fn set_yard(&self, yard: ArcYard) -> Self {
		State {
			era: self.era,
			yard,
			back_to_front: self.back_to_front.to_vec(),
			report_link: self.report_link.to_owned(),
		}
	}
}

pub(crate) enum Action {
	SetYard { era: usize, yard: ArcYard },
	PushFront(Arc<dyn YardObservable>),
	PopFront,
}

impl story::Spark for YardStack {
	type State = State;
	type Action = Action;
	type Report = ();

	fn create(&self, link: Option<Link<()>>) -> Self::State {
		State {
			era: 0,
			yard: yard::empty(),
			back_to_front: Vec::new(),
			report_link: link,
		}
	}

	fn flow(ctx: &impl Flow<Self::State, Self::Action>, action: Self::Action) -> AfterFlow<Self::State> {
		match action {
			Action::PopFront => {
				if ctx.state().back_to_front.len() <= 1 {
					if let Some(report_link) = &ctx.state().report_link {
						report_link.send(())
					}
					AfterFlow::Ignore
				} else {
					let state = ctx.state().pop_front();
					spawn_yard_builder(&state.back_to_front, state.era, ctx.link().clone());
					AfterFlow::ReviseQuietly(state)
				}
			}
			Action::PushFront(front) => {
				let state = ctx.state().push_front(front);
				spawn_yard_builder(&state.back_to_front, state.era, ctx.link().clone());
				AfterFlow::ReviseQuietly(state)
			}
			Action::SetYard { era, yard } => {
				if era == ctx.state().era {
					let state = ctx.state().set_yard(yard);
					AfterFlow::Revise(state)
				} else {
					AfterFlow::Ignore
				}
			}
		}
	}

	fn yard(vision: &Self::State, _link: &Link<Self::Action>) -> Option<ArcYard> {
		Some(vision.yard.to_owned())
	}
}

fn spawn_yard_builder(back_to_front: &Vec<Arc<dyn YardObservable>>, era: usize, link: Link<Action>) {
	let back = back_to_front.first().unwrap().to_owned();
	let front = (&back_to_front[1..]).to_vec().into_iter().fold(back, overlay);
	let yards = front.subscribe().unwrap();
	thread::spawn(move || {
		for yard in yards {
			link.send(Action::SetYard { era, yard })
		}
	});
}
