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

use yui::{ActionContext, App, Link, Projector};
use yui::{AfterAction, ArcYard, Before, Cling, Confine, Pack, Padding, story, yard};
use yui::palette::{FillColor, StrokeColor};
use yui::tabbar::tabbar_yard;

fn main() -> Result<(), Box<dyn Error>> {
	WriteLogger::init(LevelFilter::Info, Config::default(), File::create("yui.log").unwrap()).unwrap();
	info!("Demo");
	let app = App::start::<Demo>()?;
	Projector::project_app(&app)?;
	Ok(())
}

#[derive(Clone, Debug)]
pub struct Demo {
	main_tab: MainTab
}

impl story::Plot for Demo {
	type V = Self;
	type A = Action;

	fn create() -> Self::V {
		Demo { main_tab: MainTab::Button }
	}

	fn action(ctx: &impl ActionContext<Self::V, Self::A>, action: Action) -> AfterAction<Demo> {
		match action {
			Action::ShowTab(tab) => {
				AfterAction::Revise(Demo { main_tab: tab })
			}
			Action::OpenDialog => {
				ctx.start_prequel::<Demo>();
				AfterAction::Ignore
			}
			Action::CloseDialog => {
				ctx.end_prequel();
				AfterAction::Ignore
			}
		}
	}

	fn yard(vision: &Demo, link: &Link<Action>) -> Option<ArcYard> {
		let Demo { main_tab } = vision;
		let select_tab = link.callback(|index| {
			Action::ShowTab(match index {
				0 => MainTab::Button,
				_ => MainTab::TextField,
			})
		});
		let yard = match main_tab {
			MainTab::Button => {
				let active_tab = 0;
				let focused_button = yard::button("Close Dialog", link.callback(|_| Action::CloseDialog));
				let enabled_button = yard::button("Open  Dialog", link.callback(|_| Action::OpenDialog));
				let trellis = yard::trellis(3, 2, vec![focused_button, enabled_button]);
				let content = trellis.confine(32, 8, Cling::CenterMiddle)
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
		Some(yard)
	}
}


#[derive(Clone, Debug)]
pub enum Action {
	ShowTab(MainTab),
	OpenDialog,
	CloseDialog,
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
