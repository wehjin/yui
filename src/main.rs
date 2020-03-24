#[macro_use]
extern crate log;
extern crate ncurses;
extern crate simplelog;

use std::collections::HashMap;
use std::fs::File;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use log::LevelFilter;
use simplelog::{Config, WriteLogger};

use yui::*;

use crate::yui::button::button_yard;
use crate::yui::empty::empty_yard;
use crate::yui::fill::fill_yard;
use crate::yui::label::label_yard;
use crate::yui::palette::{FillColor, StrokeColor};
use crate::yui::tabbar::tabbar_yard;
use crate::yui_curses::Projector;

mod yui;
mod yui_curses;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum MainTab {
	Button,
	TextField,
}

#[derive(Clone, Debug)]
struct MainVision {
	main_tab: MainTab
}

#[derive(Clone, Debug)]
enum MainAction {
	Subscribe(i32, Sender<MainVision>)
}

struct Story {
	action_sender: Sender<MainAction>
}

impl Story {
	fn new() -> Self {
		let (action_sender, action_receiver) = channel();
		thread::spawn(move || {
			let mut vision_senders: HashMap<i32, Sender<MainVision>> = HashMap::new();
			let vision = MainVision { main_tab: MainTab::Button };
			loop {
				let action = action_receiver.recv().unwrap();
				match action {
					MainAction::Subscribe(subscriber_id, vision_sender) => {
						vision_sender.send(vision.clone()).unwrap();
						vision_senders.insert(subscriber_id, vision_sender);
					}
				}
			}
		});
		Story { action_sender }
	}

	fn subscribe(&self) -> Receiver<MainVision> {
		let (send_vision, receive_vision) = channel::<MainVision>();
		self.action_sender.send(MainAction::Subscribe(rand::random(), send_vision)).unwrap();
		receive_vision
	}
}

fn main() {
	WriteLogger::init(LevelFilter::Info, Config::default(), File::create("yui.log").unwrap()).unwrap();

	let story = Story::new();
	Projector::run_blocking(move |ctx| {
		let tab_labels = vec!["Button", "Text Field", "About Us"];
		let visions = story.subscribe();
		loop {
			let MainVision { main_tab } = visions.recv().unwrap();
			match main_tab {
				MainTab::Button => {
					let active_tab = 0;
					let focused_button = button_yard("Focused");
					let enabled_button = button_yard("Enabled");
					let button_pole = enabled_button
						.pack_top(1, empty_yard())
						.pack_top(1, focused_button);
					let content = button_pole.confine(32, 3, Cling::CenterMiddle)
						.pad(1)
						.before(fill_yard(FillColor::Background));
					ctx.set_yard(content
						.pack_top(3, tabbar_yard(&tab_labels, active_tab))
						.pack_top(3, app_bar())
					);
				}
				MainTab::TextField => {
					let active_tab = 1;
					let label = label_yard("Text Field", StrokeColor::BodyOnBackground, Cling::CenterMiddle);
					let content = label
						.pad(1)
						.before(fill_yard(FillColor::Background));
					ctx.set_yard(content
						.pack_top(3, tabbar_yard(&tab_labels, active_tab))
						.pack_top(3, app_bar())
					);
				}
			}
		}
	});
}

fn app_bar() -> ArcYard {
	let tool_bar = label_yard("Components", StrokeColor::BodyOnPrimary, Cling::Custom { x: 0.0, y: 0.0 });
	let header_row = tool_bar.pad(1).before(fill_yard(FillColor::Primary));
	header_row
}
