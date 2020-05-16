use std::sync::Arc;
use std::thread;

use crate::{AfterRoll, ArcYard, Link, RollContext, story, yard};
use crate::yard::{overlay, YardObservable};

pub(crate) struct YardStack;

#[derive(Clone)]
pub(crate) struct Vision {
	era: usize,
	yard: ArcYard,
	back_to_front: Vec<Arc<dyn YardObservable>>,
}

pub(crate) enum Action {
	SetYard { era: usize, yard: ArcYard },
	PushFront(Arc<dyn YardObservable>),
	PopFront,
}

impl story::Wheel for YardStack {
	type State = Vision;
	type Action = Action;

	fn build() -> Self::State {
		Vision { era: 0, yard: yard::empty(), back_to_front: Vec::new() }
	}

	fn roll(ctx: &impl RollContext<Self::State, Self::Action>, action: Self::Action) -> AfterRoll<Self::State> {
		match action {
			Action::PopFront => {
				if ctx.state().back_to_front.len() <= 1 {
					AfterRoll::Ignore
				} else {
					let mut back_to_front = ctx.state().back_to_front.to_vec();
					back_to_front.pop();
					let yard = ctx.state().yard.to_owned();
					let era = ctx.state().era + 1;
					spawn_yard_builder(&back_to_front, era, ctx.link().clone());
					AfterRoll::TurnQuietly(Vision { era, yard, back_to_front })
				}
			}
			Action::PushFront(front) => {
				let mut back_to_front = ctx.state().back_to_front.to_vec();
				back_to_front.push(front);
				let yard = ctx.state().yard.to_owned();
				let era = ctx.state().era + 1;
				spawn_yard_builder(&back_to_front, era, ctx.link().clone());
				AfterRoll::TurnQuietly(Vision { era, yard, back_to_front })
			}
			Action::SetYard { era, yard } => {
				if era == ctx.state().era {
					let back_to_front = ctx.state().back_to_front.to_vec();
					AfterRoll::Turn(Vision { era, yard, back_to_front })
				} else {
					AfterRoll::Ignore
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
