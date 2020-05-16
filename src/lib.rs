#[macro_use]
extern crate log;
extern crate ncurses;
extern crate simplelog;
extern crate stringedit;


pub use app::App;
pub use story::*;
pub use yard::ArcYard;
pub use yui::*;
pub use yui_curses::*;

pub mod yard;
mod yui;
mod yui_curses;
pub mod story;

mod app {
	use std::error::Error;
	use std::sync::Arc;
	use std::sync::mpsc::Receiver;

	use crate::{ArcYard, Link, story, Story, Wheel};
	use crate::app::yard_stack::YardStack;
	use crate::yard::{YardObservable, YardObservableSource};

	pub struct Edge {
		link: Link<yard_stack::Action>
	}

	impl Clone for Edge {
		fn clone(&self) -> Self { Edge { link: self.link.clone() } }
	}

	impl Edge {
		pub fn start_dialog<W: Wheel>(&self) -> Story<W> {
			let story = W::launch(Some(self.clone()));
			let yards = story.yards();
			self.link.send(yard_stack::Action::PushFront(yards));
			story
		}

		pub fn end_dialog(&self) {
			self.link.send(yard_stack::Action::PopFront);
		}
	}

	pub struct App {
		front_yards: Arc<dyn YardObservable>
	}

	impl App {
		pub fn subscribe_yards(&self) -> Result<Receiver<ArcYard>, Box<dyn Error>> {
			self.front_yards.subscribe()
		}
		pub fn start<T: story::Wheel>() -> Result<Self, Box<dyn Error>> {
			let yard_stack = YardStack::launch(None);
			let edge = Edge { link: yard_stack.link() };
			let teller_story = T::launch(Some(edge));
			let teller_yards = teller_story.yards();
			yard_stack.link().send(yard_stack::Action::PushFront(teller_yards));

			let yard_stack_yards = yard_stack.yards();
			let app = App { front_yards: yard_stack_yards };
			Ok(app)
		}
	}

	pub(crate) mod yard_stack {
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
						if ctx.vision().back_to_front.len() <= 1 {
							AfterRoll::Ignore
						} else {
							let mut back_to_front = ctx.vision().back_to_front.to_vec();
							back_to_front.pop();
							let yard = ctx.vision().yard.to_owned();
							let era = ctx.vision().era + 1;
							spawn_yard_builder(&back_to_front, era, ctx.link().clone());
							AfterRoll::TurnQuietly(Vision { era, yard, back_to_front })
						}
					}
					Action::PushFront(front) => {
						let mut back_to_front = ctx.vision().back_to_front.to_vec();
						back_to_front.push(front);
						let yard = ctx.vision().yard.to_owned();
						let era = ctx.vision().era + 1;
						spawn_yard_builder(&back_to_front, era, ctx.link().clone());
						AfterRoll::TurnQuietly(Vision { era, yard, back_to_front })
					}
					Action::SetYard { era, yard } => {
						if era == ctx.vision().era {
							let back_to_front = ctx.vision().back_to_front.to_vec();
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
	}
}