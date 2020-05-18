#[macro_use]
extern crate log;
extern crate ncurses;
extern crate simplelog;
extern crate stringedit;
extern crate yui;

use std::error::Error;
use std::fs::File;
use std::iter::FromIterator;

use log::LevelFilter;
use simplelog::{Config, WriteLogger};

use yui::{app, Link, RollContext};
use yui::{AfterRoll, ArcYard, Before, Cling, Confine, Pack, Padding, story, yard};
use yui::palette::{FillColor, StrokeColor};
use yui::StringEdit;
use yui::tabbar::tabbar_yard;

fn main() -> Result<(), Box<dyn Error>> {
	WriteLogger::init(LevelFilter::Info, Config::default(), File::create("yui.log").unwrap()).unwrap();
	info!("Demo");
	app::run::<Demo>(None)
}

#[derive(Clone, Debug)]
pub struct Demo {
	main_tab: MainTab,
	edit: StringEdit,
}

impl Demo {
	fn with_edit(&self, action: stringedit::Action) -> Self {
		let edit = self.edit.edit(action);
		Demo { main_tab: self.main_tab, edit }
	}
	fn with_tab(&self, main_tab: MainTab) -> Self {
		Demo { main_tab, edit: self.edit.clone() }
	}
	fn new() -> Self {
		Demo { main_tab: MainTab::Button, edit: StringEdit::empty() }
	}
}

impl story::Wheel for Demo {
	type State = Self;
	type Action = Action;
	type Report = ();

	fn build(_link: Option<Link<Self::Report>>) -> Self::State { Demo::new() }

	fn roll(ctx: &impl RollContext<Self::State, Self::Action>, action: Action) -> AfterRoll<Demo> {
		match action {
			Action::StringEdit(edit) => AfterRoll::Revise(ctx.state().with_edit(edit)),
			Action::ShowTab(tab) => AfterRoll::Revise(ctx.state().with_tab(tab)),
			Action::OpenDialog => {
				ctx.start_prequel::<Demo>();
				AfterRoll::Ignore
			}
			Action::CloseDialog => {
				ctx.end_prequel();
				AfterRoll::Ignore
			}
		}
	}

	fn yard(vision: &Demo, link: &Link<Action>) -> Option<ArcYard> {
		let Demo { main_tab, edit } = vision;
		let select_tab = link.callback(|index| {
			Action::ShowTab(match index {
				0 => MainTab::Button,
				1 => MainTab::TextField,
				2 => MainTab::QuadLabel,
				_ => unimplemented!("No tab for index {}", index)
			})
		});
		let yard = match main_tab {
			MainTab::Button => {
				let trellis = yard::trellis(3, 2, vec![
					yard::button("Open  Dialog", link.callback(|_| Action::OpenDialog)),
					yard::button("Close", link.callback(|_| Action::CloseDialog)),
				]);
				let content = trellis.confine(32, 8, Cling::Center)
					.pad(1)
					.before(yard::fill(FillColor::Background));
				tab_page(content, 0, select_tab)
			}
			MainTab::TextField => {
				let link = link.clone();
				let trellis = yard::trellis(3, 1, vec![
					yard::label(
						&String::from_iter(edit.chars.to_vec()),
						StrokeColor::BodyOnBackground,
						Cling::Left,
					),
					yard::textfield(
						1932,
						"Label".into(),
						edit.clone(),
						move |new_edit| link.send(Action::StringEdit(new_edit)),
					),
				]);
				let content =
					trellis
						.confine(50, 7, Cling::Center)
						.pad(1)
						.before(yard::fill(FillColor::Background));
				tab_page(content, 1, select_tab)
			}
			MainTab::QuadLabel => {
				let quad_label = yard::quad_label(
					"Title",
					"sub-title",
					"1 Value",
					"2 sub-value",
					15,
					FillColor::Background,
				);
				let content = quad_label.pad(1).confine(40, 4, Cling::Center);
				tab_page(content, 2, select_tab)
			}
		};
		Some(yard)
	}
}


#[derive(Clone, Debug)]
pub enum Action {
	StringEdit(stringedit::Action),
	ShowTab(MainTab),
	OpenDialog,
	CloseDialog,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum MainTab {
	Button,
	TextField,
	QuadLabel,
}

static TABS: &'static [(i32, &str)] = &[
	(1, "Button"),
	(2, "Text Field"),
	(3, "Quad Label"),
];

fn app_bar() -> ArcYard {
	let tool_bar = yard::title("Components", StrokeColor::BodyOnPrimary, Cling::Custom { x: 0.0, y: 0.0 });
	let header_row = tool_bar.pad(1).before(yard::fill(FillColor::Primary));
	header_row
}

fn tab_page(content: ArcYard, active_tab_index: usize, select_tab: impl Fn(usize) + Send + Sync + 'static) -> ArcYard {
	content
		.pack_top(3, tabbar_yard(TABS, active_tab_index, select_tab))
		.pack_top(3, app_bar())
}
