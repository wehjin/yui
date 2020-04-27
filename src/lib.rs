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

	use crate::{ArcYard, Link, story, Story, Teller};
	use crate::app::yard_stack::YardStack;
	use crate::yard::{YardObservable, YardObservableSource};

	pub struct AppContext {
		link: Link<yard_stack::Action>
	}

	impl Clone for AppContext {
		fn clone(&self) -> Self { AppContext { link: self.link.clone() } }
	}

	impl AppContext {
		pub fn start_dialog<T: Teller>(&self) -> Story<T> {
			let story = T::begin_story(Some(self.clone()));
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
		pub fn start<T: story::Teller>() -> Result<Self, Box<dyn Error>> {
			let yard_stack = YardStack::begin_story(None);
			let app_context = AppContext { link: yard_stack.link() };
			let teller_story = T::begin_story(Some(app_context));
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
			SetYard { era: usize, yard: ArcYard },
			PushFront(Arc<dyn YardObservable>),
			PopFront,
		}

		impl story::Teller for YardStack {
			type V = Vision;
			type A = Action;

			fn create() -> Self::V {
				Vision { era: 0, yard: yard::empty(), back_to_front: Vec::new() }
			}

			fn update(ctx: &impl UpdateContext<Self::V, Self::A>, action: Self::A) -> AfterUpdate<Self::V> {
				match action {
					Action::PopFront => {
						if ctx.vision().back_to_front.len() <= 1 {
							AfterUpdate::Ignore
						} else {
							let mut back_to_front = ctx.vision().back_to_front.to_vec();
							back_to_front.pop();
							let yard = ctx.vision().yard.to_owned();
							let era = ctx.vision().era + 1;
							spawn_yard_builder(&back_to_front, era, ctx.link().clone());
							AfterUpdate::ReviseQuietly(Vision { era, yard, back_to_front })
						}
					}
					Action::PushFront(front) => {
						let mut back_to_front = ctx.vision().back_to_front.to_vec();
						back_to_front.push(front);
						let yard = ctx.vision().yard.to_owned();
						let era = ctx.vision().era + 1;
						spawn_yard_builder(&back_to_front, era, ctx.link().clone());
						AfterUpdate::ReviseQuietly(Vision { era, yard, back_to_front })
					}
					Action::SetYard { era, yard } => {
						if era == ctx.vision().era {
							let back_to_front = ctx.vision().back_to_front.to_vec();
							AfterUpdate::Revise(Vision { era, yard, back_to_front })
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