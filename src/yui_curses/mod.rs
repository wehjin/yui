use std::error::Error;
use std::sync::mpsc::{channel, Receiver};
use std::thread;

use ncurses::*;

use keyboard::Keyboard;
pub(crate) use screen::ScreenAction;

use crate::{Sendable, SenderLink, trigger, Trigger};
use crate::yard::ArcYard;

mod screen;
mod keyboard;
pub mod spot;

pub struct Projector {
	set_yard_fn: Box<dyn Fn(ArcYard)>,
}

#[derive(Debug, Clone)]
pub enum ProjectorReport {
	Running { refresh_trigger: Trigger }
}

impl Sendable for ProjectorReport {}

impl Projector {
	fn new(on_yard: impl Fn(ArcYard) + 'static) -> Self {
		Projector { set_yard_fn: Box::new(on_yard) }
	}
}

impl Projector {
	pub fn set_yard(&self, yard: ArcYard) {
		(*self.set_yard_fn)(yard)
	}

	pub fn project_yards(yards: Receiver<Option<ArcYard>>, report_link: SenderLink<ProjectorReport>) -> Result<(), Box<dyn Error>> {
		let (stop_tx, stop_rx) = channel();
		setlocale(LcCategory::all, "en_US.UTF-8");
		initscr();
		if !has_colors() {
			endwin();
			println!("Your terminal does not support color");
			std::process::exit(1);
		}
		let screen_link = screen::connect();
		ProjectorReport::Running { refresh_trigger: trigger(ScreenAction::ResizeRefresh, &screen_link) }
			.send(&report_link);
		{
			let screen_link = screen_link.clone();
			thread::Builder::new().name("run_blocking".into()).spawn(move || {
				let projector = Projector::new(move |yard| {
					ScreenAction::SetYard(yard).send2(&screen_link, "send SetYard to screen");
				});
				for yard in &yards {
					match yard {
						Some(yard) => projector.set_yard(yard),
						None => break,
					}
				}
				stop_tx.send(()).expect("send () to stop_tx");
			}).expect("spawn");
		}
		Keyboard::read_blocking(screen_link.clone(), stop_rx);
		Ok(())
	}
}

