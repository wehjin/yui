#[macro_use]
extern crate log;
extern crate ncurses;
extern crate simplelog;
extern crate stringedit;
extern crate yui;

use std::error::Error;
use std::fs::File;

use log::LevelFilter;
use simplelog::{Config, WriteLogger};

use yui::{App, Link, Projector, UpdateContext};
use yui::{AfterUpdate, ArcYard, Before, Cling, Confine, Pack, Padding, story, yard};
use yui::button::button_yard;
use yui::empty::empty_yard;
use yui::palette::{FillColor, StrokeColor};
use yui::tabbar::tabbar_yard;

fn main() -> Result<(), Box<dyn Error>> {
	WriteLogger::init(LevelFilter::Info, Config::default(), File::create("yui.log").unwrap()).unwrap();
	info!("Demo");
	let app = App::start::<Demo>()?;
	Projector::project_app(&app)?;
	Ok(())
}

pub struct Demo;

impl story::Teller for Demo {
	type V = DemoVision;
	type A = DemoAction;

	fn create() -> DemoVision {
		DemoVision { main_tab: MainTab::Button }
	}

	fn update(_ctx: &impl UpdateContext<Self::V>, action: DemoAction) -> AfterUpdate<DemoVision> {
		match action {
			DemoAction::ShowTab(tab) => AfterUpdate::Revise(DemoVision { main_tab: tab }),
		}
	}

	fn yard(vision: &DemoVision, link: &Link<DemoAction>) -> Option<ArcYard> {
		let DemoVision { main_tab } = vision;
		let select_tab = link.callback(|index| {
			DemoAction::ShowTab(match index {
				0 => MainTab::Button,
				_ => MainTab::TextField,
			})
		});
		let yard = match main_tab {
			MainTab::Button => {
				let active_tab = 0;
				let focused_button = button_yard("Focused");
				let enabled_button = button_yard("Enabled");
				let button_pole = enabled_button
					.pack_top(1, empty_yard())
					.pack_top(1, focused_button);
				let content = button_pole.confine(32, 3, Cling::CenterMiddle)
					.pad(1)
					.before(yard::fill(FillColor::Background));
				content
					.pack_top(3, tabbar_yard(TABS, active_tab, select_tab))
					.pack_top(3, app_bar())
			}
			MainTab::TextField => {
				let active_tab = 1;
				let textfield = yard::textfield("Label");
				let content = textfield
					.confine(50, 3, Cling::CenterMiddle)
					.pad(1)
					.before(yard::fill(FillColor::Background));
				content
					.pack_top(3, tabbar_yard(TABS, active_tab, select_tab))
					.pack_top(3, app_bar())
			}
		};
		//let yard = yard.fade((60, 38));
		Some(yard)
	}
}

#[derive(Clone, Debug)]
pub struct DemoVision {
	main_tab: MainTab
}

#[derive(Clone, Debug)]
pub enum DemoAction {
	ShowTab(MainTab),
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum MainTab {
	Button,
	TextField,
}

fn app_bar() -> ArcYard {
	let tool_bar = yard::label("Components", StrokeColor::BodyOnPrimary, Cling::Custom { x: 0.0, y: 0.0 });
	let header_row = tool_bar.pad(1).before(yard::fill(FillColor::Primary));
	header_row
}

static TABS: &'static [(i32, &str)] = &[
	(1, "Button"),
	(2, "Text Field")
];
