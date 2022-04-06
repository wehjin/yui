use std::error::Error;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use ncurses::{endwin, has_colors, initscr, LcCategory, setlocale};

use crate::{ArcYard, ScreenAction, Sendable, Spark, Story, story, Trigger};
use crate::app::SimpleEdge;
use crate::yard::YardPublisher;
use crate::yui_curses::{screen, spawn_screen_feeder};
use crate::yui_curses::keyboard::Keyboard;

pub fn run(spark: impl Spark + Send + 'static) -> Result<(), Box<dyn Error>> {
	let console = Console::connect();
	let story = story::spark(spark, Some(SimpleEdge::new(console.refresh_trigger().clone())), None);
	console.run_story(story)?;
	Ok(())
}

pub struct Console {
	screen_link: Sender<ScreenAction>,
	refresh_trigger: Trigger,
}

impl Console {
	pub fn connect() -> Self {
		setlocale(LcCategory::all, "en_US.UTF-8");
		initscr();
		if !has_colors() {
			endwin();
			println!("Your terminal does not support color");
			std::process::exit(1);
		}
		let screen_link = screen::connect();
		let refresh_trigger = ScreenAction::ResizeRefresh.into_trigger(&screen_link);
		Console { screen_link, refresh_trigger }
	}
	pub fn refresh_trigger(&self) -> &Trigger { &self.refresh_trigger }
	pub fn run(&self, yard_source: Receiver<Option<ArcYard>>) {
		let (done_tx, done_rx) = channel();
		spawn_screen_feeder(yard_source, done_tx, &self.screen_link);
		Keyboard::read_blocking(self.screen_link.clone(), done_rx);
	}
	pub fn run_story<Sp: Spark + 'static>(&self, story: Story<Sp>) -> Result<(), Box<dyn Error>> {
		let yard_source = {
			let (opt_yard_link, opt_yard_source) = channel();
			let yard_source = story.subscribe()?;
			thread::spawn(move || for yard in yard_source {
				opt_yard_link.send(Some(yard)).expect("Send opt-yard");
			});
			opt_yard_source
		};
		self.run(yard_source);
		Ok(())
	}
}
