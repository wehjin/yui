#[macro_use]
extern crate log;
extern crate ncurses;
extern crate simplelog;
extern crate stringedit;
extern crate yui;

use std::fs::File;

use log::LevelFilter;
use simplelog::{Config, WriteLogger};

use yui::*;

use crate::Action::ShowTab;
use crate::yui::button::button_yard;
use crate::yui::empty::empty_yard;
use crate::yui::palette::{FillColor, StrokeColor};
use crate::yui::Projector;
use crate::yui::tabbar::tabbar_yard;
use crate::yui::yard;

fn main() {
	WriteLogger::init(LevelFilter::Info, Config::default(), File::create("yui.log").unwrap()).unwrap();
	info!("Demo");
	let story = Demo::story();

	let tabs = vec![
		(rand::random(), "Button"),
		(rand::random(), "Text Field")
	];
	Projector::run_blocking(move |ctx| {
		let visions = story.subscribe(rand::random()).unwrap();
		loop {
			let select_tab = story.callback(|index| {
				Action::ShowTab(match index {
					0 => MainTab::Button,
					_ => MainTab::TextField,
				})
			});
			let Vision { main_tab } = visions.recv().unwrap();
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
						.before(fill(FillColor::Background));
					ctx.set_yard(content
						.pack_top(3, tabbar_yard(&tabs, active_tab, select_tab))
						.pack_top(3, app_bar()));
				}
				MainTab::TextField => {
					let active_tab = 1;
					let textfield = yard::textfield("Label");
					let content = textfield
						.confine(50, 3, Cling::CenterMiddle)
						.pad(1)
						.before(fill(FillColor::Background));
					ctx.set_yard(content
						.pack_top(3, tabbar_yard(&tabs, active_tab, select_tab))
						.pack_top(3, app_bar())
					);
				}
			}
		}
	});
}

struct Demo;

impl StoryTeller for Demo {
	type V = Vision;
	type A = Action;

	fn create() -> Vision {
		Vision { main_tab: MainTab::Button }
	}

	fn update(_vision: &Vision, action: Action) -> AfterUpdate<Vision> {
		match action {
			ShowTab(tab) => AfterUpdate::Revise(Vision { main_tab: tab }),
		}
	}
}

#[derive(Clone, Debug)]
struct Vision {
	main_tab: MainTab
}

#[derive(Clone, Debug)]
enum Action {
	ShowTab(MainTab),
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum MainTab {
	Button,
	TextField,
}

fn app_bar() -> ArcYard {
	let tool_bar = yard::label("Components", StrokeColor::BodyOnPrimary, Cling::Custom { x: 0.0, y: 0.0 });
	let header_row = tool_bar.pad(1).before(fill(FillColor::Primary));
	header_row
}
