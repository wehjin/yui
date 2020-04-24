#[macro_use]
extern crate log;
extern crate ncurses;
extern crate simplelog;
extern crate stringedit;


pub use story::{AfterUpdate, Story};
pub use story::Teller as StoryTeller;
pub use yui::*;
pub use yui_curses::*;

mod yui;
mod yui_curses;

pub mod story;