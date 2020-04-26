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
	use std::sync::mpsc::Receiver;

	use crate::{ArcYard, story};
	use crate::yard::{YardObservable, YardObservableSource};

	pub struct App {
		front_yard: Box<dyn YardObservable>
	}

	impl App {
		pub fn subscribe_yards(&self) -> Result<Receiver<ArcYard>, Box<dyn Error>> {
			self.front_yard.subscribe()
		}
		pub fn start<T: story::Teller + 'static>() -> Result<Self, Box<dyn Error>> {
			let story = T::begin_story();
			let app = App { front_yard: story.yards() };
			Ok(app)
		}
	}
}