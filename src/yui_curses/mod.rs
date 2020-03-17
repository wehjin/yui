use std::thread;

use ncurses::*;

use crate::yui::ArcYard;
use crate::yui_curses::keyboard::Keyboard;
use crate::yui_curses::screen::{CursesScreen, ScreenAction};

mod screen;
mod keyboard;

pub struct Projector {
	set_yard_fn: Box<dyn Fn(ArcYard)>
}

impl Projector {
	pub fn set_yard(&self, yard: ArcYard) {
		(*self.set_yard_fn)(yard)
	}

	pub fn run_blocking(block: impl Fn(Projector) + Sync + Send + 'static) {
		initscr();
		if !has_colors() {
			endwin();
			println!("Your terminal does not support color");
			std::process::exit(1);
		}
		let screen_tx = CursesScreen::start();
		let block_tx = screen_tx.clone();
		thread::spawn(move || {
			let projector = Projector {
				set_yard_fn: Box::new(move |yard| block_tx.send(ScreenAction::SetYard(yard)).unwrap())
			};
			block(projector);
		});

		Keyboard::read_blocking(screen_tx)
	}
}
