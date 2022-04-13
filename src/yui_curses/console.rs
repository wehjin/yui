use std::error::Error;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use ncurses::{endwin, has_colors, initscr, LcCategory, setlocale};

use crate::{ArcYard, Link, ScreenAction, Sendable, Spark, Story, StoryVerse, Trigger};
use crate::pod::Pod;
use crate::pod_verse::PodVerse;
use crate::story_id::StoryId;
use crate::yard::YardPublisher;
use crate::yui_curses::{screen, spawn_screen_feeder};
use crate::yui_curses::keyboard::Keyboard;

pub fn run_spark<S: Spark>(spark: S) where S: Send + 'static {
	let main_story_id = StoryId::new(0);
	let (story_verse, _) = StoryVerse::build(spark, main_story_id);
	let pod_verse = PodVerse::build(&story_verse, main_story_id);
	run_pod_verse(&pod_verse);
}

pub fn run_pod_verse(pod_verse: &PodVerse) {
	let console = Console::connect_pod_verse(pod_verse);
	let (done_tx, done_rx) = channel();
	pod_verse.set_done_trigger(done_tx);
	Keyboard::read_blocking(console.screen_link.clone(), done_rx);
}

pub struct Console {
	screen_link: Sender<ScreenAction>,
	refresh_trigger: Trigger,
}

impl Console {
	pub fn connect(pod: impl Pod + Send + 'static) -> Self {
		Self::init_connect();
		let screen_link = screen::connect(pod);
		Self::finish_connect(screen_link)
	}
	fn init_connect() {
		setlocale(LcCategory::all, "en_US.UTF-8");
		initscr();
		if !has_colors() {
			endwin();
			println!("Your terminal does not support color");
			std::process::exit(1);
		}
	}
	fn finish_connect(screen_link: Sender<ScreenAction>) -> Console {
		let refresh_trigger = ScreenAction::ResizeRefresh.into_trigger(&screen_link);
		refresh_trigger.send(());
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
	fn connect_pod_verse(pod_verse: &PodVerse) -> Self {
		Self::init_connect();
		let screen_link = screen::connect_pod_verse(pod_verse.clone());
		Self::finish_connect(screen_link)
	}
}

