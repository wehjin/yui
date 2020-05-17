use std::error::Error;
use std::sync::mpsc::{Receiver, sync_channel};
use std::thread;

use ncurses::*;

use keyboard::Keyboard;
use screen::{CursesScreen, ScreenAction};

use crate::yard::ArcYard;

mod screen;
mod keyboard;

pub struct Projector {
	set_yard_fn: Box<dyn Fn(ArcYard)>
}

impl Projector {
	fn new(f: impl Fn(ArcYard) + 'static) -> Self {
		Projector { set_yard_fn: Box::new(f) }
	}
}

impl Projector {
	pub fn set_yard(&self, yard: ArcYard) {
		(*self.set_yard_fn)(yard)
	}

	pub fn project_yards(yards: Receiver<Option<ArcYard>>) -> Result<(), Box<dyn Error>> {
		let (stop_tx, stop_rx) = sync_channel(1);
		Self::run_blocking(stop_rx, move |ctx| {
			for yard in &yards {
				match yard {
					Some(yard) => ctx.set_yard(yard),
					None => break,
				}
			}
			stop_tx.send(()).unwrap();
		});
		Ok(())
	}

	pub fn run_blocking(stop_rx: Receiver<()>, block: impl Fn(Projector) + Send + 'static) {
		initscr();
		if !has_colors() {
			endwin();
			println!("Your terminal does not support color");
			std::process::exit(1);
		}
		let screen_tx = CursesScreen::start();
		{
			let screen_tx = screen_tx.clone();
			thread::spawn(move || {
				let projector = Projector::new(move |yard| {
					screen_tx.send(ScreenAction::SetYard(yard)).unwrap()
				});
				block(projector);
			});
		}
		Keyboard::read_blocking(screen_tx.clone(), stop_rx)
	}
}

