#[macro_use]
extern crate log;
extern crate ncurses;
extern crate simplelog;
extern crate stringedit;
extern crate unicode_width;

pub use link::*;
pub use story::*;
pub use yard::ArcYard;
pub use yui_curses::*;

pub use self::yui::*;

pub mod app;
pub mod palette;
pub mod selection_editor;
pub mod sparks;
pub mod story;
pub mod yard;

mod link;
mod yui;
mod yui_curses;


mod surface {
	/**
	* The surface type
	*/
	pub enum Type {
		Body,
		Header,
		Navigation,
	}

	pub enum Texture {
		Plain,
		Focused,
		Pressed,
	}
}