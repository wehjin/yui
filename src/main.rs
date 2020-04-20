#[macro_use]
extern crate log;
extern crate ncurses;
extern crate simplelog;
extern crate stringedit;

use std::collections::HashMap;
use std::fs::File;
use std::sync::mpsc::{channel, Receiver, Sender, sync_channel, SyncSender};
use std::thread;

use log::LevelFilter;
use simplelog::{Config, WriteLogger};

use yui::*;

use crate::MainAction::{ShowTab, Subscribe};
use crate::yui::button::button_yard;
use crate::yui::empty::empty_yard;
use crate::yui::fill::fill_yard;
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
	Subscribe(i32, Sender<MainVision>),
	ShowTab(MainTab),
}

struct Story {
	send_actions: SyncSender<MainAction>
}

impl Story {
	fn new() -> Self {
		let (send_actions, action_receiver) = sync_channel(100);
		thread::spawn(move || {
			let mut vision_senders: HashMap<i32, Sender<MainVision>> = HashMap::new();
			fn post_vision(vision: MainVision, vision_senders: &HashMap<i32, Sender<MainVision>>) {
				vision_senders.iter().for_each(|(_, sender)| {
					sender.send(vision.clone()).unwrap()
				})
			}
			let mut vision = MainVision { main_tab: MainTab::Button };
			loop {
				let action = action_receiver.recv().unwrap();
				match action {
					Subscribe(subscriber_id, vision_sender) => {
						vision_sender.send(vision.clone()).unwrap();
						vision_senders.insert(subscriber_id, vision_sender);
					}

					ShowTab(tab) => {
						vision = MainVision { main_tab: tab, ..vision };
						post_vision(vision.clone(), &vision_senders)
					}
				}
			}
		});
		Story { send_actions }
	}

	fn subscribe(&self) -> Receiver<MainVision> {
		let (send_vision, receive_vision) = channel::<MainVision>();
		self.send_actions.send(MainAction::Subscribe(rand::random(), send_vision)).unwrap();
		receive_vision
	}

	fn select_tab(&self, into_tab: impl Fn(usize) -> MainTab + Send + Sync) -> impl Fn(usize) + Send + Sync {
		let send_actions = self.send_actions.clone();
		move |index| {
			let tab = into_tab(index);
			send_actions.send(ShowTab(tab)).unwrap()
		}
	}
}

fn main() {
	WriteLogger::init(LevelFilter::Info, Config::default(), File::create("yui.log").unwrap()).unwrap();

	let story = Story::new();
	Projector::run_blocking(move |ctx| {
		let tabs = vec![
			(rand::random(), "Button"),
			(rand::random(), "Text Field")
		];
		let visions = story.subscribe();
		loop {
			let select_tab = story.select_tab(|index| {
				match index {
					0 => MainTab::Button,
					_ => MainTab::TextField,
				}
			});
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
						.pack_top(3, tabbar_yard(&tabs, active_tab, select_tab))
						.pack_top(3, app_bar())
					);
				}
				MainTab::TextField => {
					let active_tab = 1;
					let textfield = yard::textfield("Label");
					let content = textfield
						.confine(50, 3, Cling::CenterMiddle)
						.pad(1)
						.before(fill_yard(FillColor::Background));
					ctx.set_yard(content
						.pack_top(3, tabbar_yard(&tabs, active_tab, select_tab))
						.pack_top(3, app_bar())
					);
				}
			}
		}
	});
}

fn app_bar() -> ArcYard {
	let tool_bar = yard::label("Components", StrokeColor::BodyOnPrimary, Cling::Custom { x: 0.0, y: 0.0 });
	let header_row = tool_bar.pad(1).before(fill_yard(FillColor::Primary));
	header_row
}
