use std::error::Error;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use ncurses::*;

use keyboard::Keyboard;
pub(crate) use screen::ScreenAction;

use crate::{Sendable, SenderLink, trigger, Trigger};
use crate::yard::ArcYard;

mod screen;
mod keyboard;
pub mod spot;

#[derive(Debug, Clone)]
pub enum ProjectorReport {
	Ready { refresh_trigger: Trigger }
}

impl Sendable for ProjectorReport {}

pub fn start_projector(yard_source: Receiver<Option<ArcYard>>, report_link: SenderLink<ProjectorReport>) -> Result<(), Box<dyn Error>> {
	let (done_tx, done_rx) = channel();
	setlocale(LcCategory::all, "en_US.UTF-8");
	initscr();
	if !has_colors() {
		endwin();
		println!("Your terminal does not support color");
		std::process::exit(1);
	}
	let screen_link = screen::connect();
	ProjectorReport::Ready { refresh_trigger: trigger(ScreenAction::ResizeRefresh, &screen_link) }.send(&report_link);
	spawn_screen_feeder(yard_source, done_tx, &screen_link);
	Keyboard::read_blocking(screen_link.clone(), done_rx);
	Ok(())
}

fn spawn_screen_feeder(yard_source: Receiver<Option<ArcYard>>, done_trigger: Sender<()>, screen_link: &Sender<ScreenAction>) {
	let screen_link = screen_link.clone();
	thread::Builder::new().name("run_blocking".into()).spawn(move || {
		for yard in &yard_source {
			if let Some(yard) = yard {
				ScreenAction::SetYard(yard).send2(&screen_link, "send SetYard to screen");
			} else {
				break;
			}
		}
		done_trigger.send(()).expect("send () to stop_tx");
	}).expect("spawn");
}

