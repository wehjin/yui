use std::error::Error;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

pub(crate) use screen::ScreenAction;

use crate::{Sendable, SenderLink, Trigger};
use crate::yard::ArcYard;

mod screen;
mod keyboard;
pub mod spot;

#[derive(Debug, Clone)]
pub enum ProjectorReport {
	Ready { refresh_trigger: Trigger }
}

impl Sendable for ProjectorReport {}

pub mod console;

pub fn run_console(_yard_source: Receiver<Option<ArcYard>>, _report_link: SenderLink<ProjectorReport>) -> Result<(), Box<dyn Error>> {
	// let console = Console::connect();
	// ProjectorReport::Ready { refresh_trigger: console.refresh_trigger().clone() }.send(&report_link);
	// console.run(yard_source);
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

