extern crate ncurses;

use ncurses::*;

use yui::*;

use crate::ycurses::{CursesScreen, KEY_EOT};
use crate::yui::button::button_yard;
use crate::yui::empty::empty_yard;
use crate::yui::fill::fill_yard;
use crate::yui::label::label_yard;
use crate::yui::palette::{FillColor, StrokeColor};

mod ycurses;
mod yui;

fn main() {
	let header =
		label_yard("Buttons", StrokeColor::PrimaryBody, Cling::Custom { x: 0.0, y: 0.0 }).pad(1)
			.before(fill_yard(FillColor::Primary));


	let button_pole =
		button_yard("Enabled").confine_height(1, Cling::CenterMiddle)
			.pack_top(1, empty_yard())
			.pack_top(1, button_yard("Focused"));
	let content =
		button_pole.confine(32, 3, Cling::CenterMiddle).pad(1)
			.before(fill_yard(FillColor::Background));
	let yard = content.pack_top(3, header);

	initscr();
	if !has_colors() {
		endwin();
		println!("Your terminal does not support color");
		std::process::exit(1);
	}

	CursesScreen::start(yard);
	raw();
	keypad(stdscr(), true);
	cbreak();
	noecho();
	let mut done = false;
	while !done {
		let ch = getch();
		match ch {
			KEY_EOT => {
				done = true
			}
			_ => {
				println!("KEY: {}", ch);
			}
		}
	}
	use_default_colors();
	endwin();
}
