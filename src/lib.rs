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

	use crate::{ArcYard, story, Teller};
	use crate::app::yard_stack::YardStack;
	use crate::yard::{YardObservable, YardObservableSource};

	pub struct App {
		front_yards: Arc<dyn YardObservable>
	}

	impl App {
		pub fn subscribe_yards(&self) -> Result<Receiver<ArcYard>, Box<dyn Error>> {
			self.front_yards.subscribe()
		}
		pub fn start<T: story::Teller + 'static>() -> Result<Self, Box<dyn Error>> {
			let teller_story = T::begin_story();
			let teller_yards = teller_story.yards();

			let yard_stack = YardStack::begin_story();
			yard_stack.link().send(yard_stack::Action::PushFront(teller_yards));

			let front_story = T::begin_story();
			let front_yards = front_story.yards();
			yard_stack.link().send(yard_stack::Action::PushFront(front_yards));
			//let yard = yard.fade((60, 38));

			let yard_stack_yards = yard_stack.yards();
			let app = App { front_yards: yard_stack_yards };
			Ok(app)
		}
	}

	pub(crate) mod yard_stack {
		use std::sync::Arc;
		use std::thread;

		use crate::{AfterUpdate, ArcYard, Link, story, UpdateContext, yard};
		use crate::yard::{overlay, YardObservable};

		pub(crate) struct YardStack;

		#[derive(Clone)]
		pub(crate) struct Vision {
			era: usize,
			yard: ArcYard,
			back_to_front: Vec<Arc<dyn YardObservable>>,
		}

		pub(crate) enum Action {
			PushFront(Arc<dyn YardObservable>),
			SetYard { era: usize, yard: ArcYard },
		}

		impl story::Teller for YardStack {
			type V = Vision;
			type A = Action;

			fn create() -> Self::V {
				Vision { era: 0, yard: yard::empty(), back_to_front: Vec::new() }
			}


			fn update(ctx: &impl UpdateContext<Self::V, Self::A>, action: Self::A) -> AfterUpdate<Self::V> {
				match action {
					Action::PushFront(front) => {
						let back_to_front = {
							let mut back_to_front = ctx.vision().back_to_front.to_vec();
							back_to_front.push(front);
							back_to_front
						};
						let back = back_to_front.first().unwrap().to_owned();
						let front = (&back_to_front[1..]).to_vec().into_iter().fold(back, overlay);
						let era = ctx.vision().era + 1;
						{
							let yards = front.subscribe().unwrap();
							let link: Link<Action> = ctx.link().clone();
							thread::spawn(move || {
								for yard in yards {
									link.send(Action::SetYard { era, yard })
								}
							});
						}
						let vision = Vision { era, yard: ctx.vision().yard.to_owned(), back_to_front };
						AfterUpdate::ReviseQuietly(vision)
					}
					Action::SetYard { era, yard } => {
						if era == ctx.vision().era {
							let vision = Vision {
								era,
								yard,
								back_to_front: ctx.vision().back_to_front.to_vec(),
							};
							AfterUpdate::Revise(vision)
						} else {
							AfterUpdate::Ignore
						}
					}
				}
			}

			fn yard(vision: &Self::V, _link: &Link<Self::A>) -> Option<ArcYard> {
				Some(vision.yard.to_owned())
			}
		}
	}
}