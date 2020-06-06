use std::sync::Arc;
use std::thread;

use crate::{AfterFlow, ArcYard, Create, Flow, Link, story, yard};
use crate::app::yard_stack::state::State;
use crate::yard::{overlay, YardPublisher};

mod state;

pub(crate) struct YardStack;

impl story::Spark for YardStack {
	type State = State;
	type Action = Action;
	type Report = ();

	fn yard(vision: &Self::State, _link: &Link<Self::Action>) -> Option<ArcYard> {
		Some(vision.yard.to_owned())
	}

	fn flow(ctx: &impl Flow<Self::State, Self::Action, Self::Report>, action: Self::Action) -> AfterFlow<Self::State> {
		match action {
			Action::Pop => {
				if ctx.state().back_to_front.len() < 2 {
					ctx.report(());
					AfterFlow::Ignore
				} else {
					let state = ctx.state().pop_front();
					spawn_yard_builder(&state.back_to_front, state.era, ctx.link().clone());
					AfterFlow::ReviseQuietly(state)
				}
			}
			Action::Push(front) => {
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

	fn create(&self, _create: &Create<Self::Action, Self::Report>) -> Self::State {
		State {
			era: 0,
			yard: yard::empty(),
			back_to_front: Vec::new(),
		}
	}
}

pub(crate) enum Action {
	SetYard { era: usize, yard: ArcYard },
	Push(Arc<dyn YardPublisher>),
	Pop,
}

fn spawn_yard_builder(back_to_front: &Vec<Arc<dyn YardPublisher>>, era: usize, link: Link<Action>) {
	let back = back_to_front.first().unwrap().to_owned();
	let front = (&back_to_front[1..]).to_vec().into_iter().fold(back, overlay);
	let yards = front.subscribe().unwrap();
	thread::spawn(move || {
		for yard in yards {
			link.send(Action::SetYard { era, yard })
		}
	});
}
