#[macro_use]
extern crate log;
extern crate ncurses;
extern crate simplelog;
extern crate stringedit;
extern crate unicode_width;

pub use link::*;
pub use render::*;
pub use story::*;
pub use story_verse::*;
pub use yard::ArcYard;
pub use yui_curses::*;

pub use crate::core::bounds::*;
use crate::palette::{FillColor, FillGrade, StrokeColor};
use crate::story_id::StoryId;

pub use self::yui::*;

mod core;
mod link;
mod yui;
mod yui_curses;

pub mod app;
pub mod palette;
pub mod sparks;
pub mod story;
pub mod story_verse;
pub mod yard;
pub mod layout;
pub(crate) mod render;
pub mod pod;
pub mod pod_verse;

#[cfg(test)]
mod tests;

pub trait DrawPad {
	fn fill(&mut self, bounds: &Bounds, color: FillColor);
	fn grade(&mut self, bounds: &Bounds, grade: FillGrade);
	fn glyph(&mut self, bounds: &Bounds, glyph: &str, color: StrokeColor);
	fn dark(&mut self, bounds: &Bounds, exclude: &Bounds);
	fn story(&mut self, bounds: &Bounds, story_id: StoryId);
}

