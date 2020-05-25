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
use stringedit::Validity;

use yui::{app, Flow, Link};
use yui::{AfterFlow, ArcYard, Before, Cling, Pack, Padding, story, yard};
use yui::palette::{FillColor, StrokeColor};
use yui::StringEdit;
use yui::tabbar::tabbar_yard;

mod tab;

fn main() -> Result<(), Box<dyn Error>> {
	WriteLogger::init(LevelFilter::Info, Config::default(), File::create("yui.log").unwrap()).unwrap();
	info!("Demo");
	app::run(Demo::new(), None)
}

#[derive(Clone, Debug)]
pub struct Demo {
	main_tab: MainTab,
	edit: StringEdit,
	value: i32,
}

impl Demo {
	fn with_value(&self, value: i32) -> Self {
		let mut new = self.clone();
		new.value = value;
		new
	}
	fn with_edit(&self, action: stringedit::Action) -> Self {
		let edit = self.edit.edit(action);
		Demo { main_tab: self.main_tab, edit, value: self.value }
	}
	fn with_tab(&self, main_tab: MainTab) -> Self {
		Demo { main_tab, edit: self.edit.clone(), value: self.value }
	}
	fn new() -> Self {
		Demo { main_tab: MainTab::Button, edit: StringEdit::empty(Validity::UnsignedInt), value: 1 }
	}
}

impl story::Spark for Demo {
	type State = Demo;
	type Action = Action;
	type Report = ();

	fn create(&self, _link: Option<Link<Self::Report>>) -> Self::State { self.clone() }

	fn flow(flow: &impl Flow<Self::State, Self::Action>, action: Action) -> AfterFlow<Demo> {
		match action {
			Action::SetValue(value) => {
				let state = flow.state().with_value(value);
				AfterFlow::Revise(state)
			}
			Action::StringEdit(edit) => AfterFlow::Revise(flow.state().with_edit(edit)),
			Action::ShowTab(tab) => AfterFlow::Revise(flow.state().with_tab(tab)),
			Action::OpenDialog => {
				flow.start_prequel(Demo::new());
				AfterFlow::Ignore
			}
			Action::CloseDialog => {
				flow.end_prequel();
				AfterFlow::Ignore
			}
		}
	}

	fn yard(state: &Demo, link: &Link<Action>) -> Option<ArcYard> {
		let Demo { main_tab, edit, value } = state;
		let select_tab = link.callback(|index| {
			Action::ShowTab(match index {
				0 => MainTab::Button,
				1 => MainTab::FormList,
				2 => MainTab::SelectionList,
				_ => unimplemented!("No tab for index {}", index)
			})
		});
		let yard = match main_tab {
			MainTab::Button => tab::button::render(link, select_tab),
			MainTab::FormList => tab::form_list::render(&edit, link, select_tab),
			MainTab::SelectionList => tab::selector_list::render(*value, link, select_tab)
		};
		Some(yard)
	}
}


#[derive(Clone, Debug)]
pub enum Action {
	SetValue(i32),
	StringEdit(stringedit::Action),
	ShowTab(MainTab),
	OpenDialog,
	CloseDialog,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum MainTab {
	Button,
	FormList,
	SelectionList,
}

static TABS: &'static [(i32, &str)] = &[
	(1, "Button"),
	(2, "Form List"),
	(3, "Selector List"),
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


