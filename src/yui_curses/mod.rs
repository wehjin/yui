use std::error::Error;
use std::sync::mpsc::{channel, Receiver};
use std::thread;

use ncurses::*;

use keyboard::Keyboard;
pub(crate) use screen::ScreenAction;

use crate::{Link, SenderLink};
use crate::yard::ArcYard;

mod screen;
mod keyboard;

pub struct Projector {
	set_yard_fn: Box<dyn Fn(ArcYard)>,
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

	pub fn project_yards(yards: Receiver<Option<ArcYard>>, enable_refresher: SenderLink<SenderLink<()>>) -> Result<(), Box<dyn Error>> {
		let (stop_tx, stop_rx) = channel();
		Self::run_blocking(stop_rx, enable_refresher, move |ctx| {
			for yard in &yards {
				match yard {
					Some(yard) => ctx.set_yard(yard),
					None => break,
				}
			}
			stop_tx.send(()).expect("send () to stop_tx");
		});
		Ok(())
	}

	pub fn run_blocking(
		stop_rx: Receiver<()>,
		enable_refresher: SenderLink<SenderLink<()>>,
		block: impl Fn(Projector) + Send + 'static,
	) {
		setlocale(LcCategory::all, "en_US.UTF-8");
		initscr();
		if !has_colors() {
			endwin();
			println!("Your terminal does not support color");
			std::process::exit(1);
		}
		let screen_link = screen::connect();
		enable_refresher.send(SenderLink::new(screen_link.clone(), |_| ScreenAction::ResizeRefresh));
		thread::Builder::new().name("run_blocking".to_string()).spawn({
			let screen_tx = screen_link.clone();
			move || {
				let projector = Projector::new(
					move |yard| screen_tx.send(ScreenAction::SetYard(yard)).expect("send SetYard to screen")
				);
				block(projector);
			}
		}).expect("spawn");
		Keyboard::read_blocking(screen_link.clone(), stop_rx);
	}
}

