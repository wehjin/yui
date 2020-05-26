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

#[derive(Debug, Clone)]
pub struct Demo {
	main_tab: MainTab,
	edit: StringEdit,
	value: i32,
	first_dialog: u32,
	next_dialog: u32,
}

impl Demo {
	fn with_value(&self, value: i32) -> Self {
		let mut next = self.clone();
		next.value = value;
		next
	}
	fn with_edit(&self, action: stringedit::Action) -> Self {
		let mut next = self.clone();
		next.edit = self.edit.edit(action);
		next
	}
	fn with_tab(&self, main_tab: MainTab) -> Self {
		let mut next = self.clone();
		next.main_tab = main_tab;
		next
	}
	fn with_dialogs(&self, first_dialog: u32, next_dialog: u32) -> Self {
		let mut next = self.clone();
		next.first_dialog = first_dialog;
		next.next_dialog = next_dialog;
		next
	}
	fn with_next_dialog(&self, next_dialog: u32) -> Self {
		let mut next = self.clone();
		next.next_dialog = next_dialog;
		next
	}
	fn new() -> Self {
		Demo {
			main_tab: MainTab::Dialog,
			edit: StringEdit::empty(Validity::UnsignedInt),
			value: 1,
			first_dialog: 1,
			next_dialog: 2,
		}
	}
}

impl story::Spark for Demo {
	type State = Demo;
	type Action = Action;
	type Report = u32;

	fn create(&self, _link: Option<Link<Self::Report>>) -> Self::State {
		self.clone()
	}

	fn flow(flow: &impl Flow<Self::State, Self::Action, Self::Report>, action: Action) -> AfterFlow<Demo> {
		match action {
			Action::SetNextDialog(dialogs) => AfterFlow::Revise(flow.state().with_next_dialog(dialogs)),
			Action::SetValue(value) => AfterFlow::Revise(flow.state().with_value(value)),
			Action::StringEdit(edit) => AfterFlow::Revise(flow.state().with_edit(edit)),
			Action::ShowTab(tab) => AfterFlow::Revise(flow.state().with_tab(tab)),
			Action::OpenDialog => {
				let state = flow.state();
				let link = flow.link().clone();
				let next_first_dialog = state.next_dialog;
				flow.start_prequel(Demo::new().with_dialogs(next_first_dialog, next_first_dialog + 1), move |next_dialog| {
					link.send(Action::SetNextDialog(next_dialog))
				});
				AfterFlow::Ignore
			}
			Action::CloseDialog => {
				flow.report(flow.state().next_dialog);
				flow.end_prequel();
				AfterFlow::Ignore
			}
		}
	}

	fn yard(state: &Demo, link: &Link<Action>) -> Option<ArcYard> {
		let Demo { main_tab, edit, value, first_dialog, next_dialog } = state;
		let select_tab = link.callback(|index| {
			Action::ShowTab(match index {
				0 => MainTab::Dialog,
				1 => MainTab::FormList,
				2 => MainTab::SelectionList,
				_ => unimplemented!("No tab for index {}", index)
			})
		});
		let yard = match main_tab {
			MainTab::Dialog => tab::button::render(*first_dialog, *next_dialog, link, select_tab),
			MainTab::FormList => tab::form_list::render(&edit, link, select_tab),
			MainTab::SelectionList => tab::selector_list::render(*value, link, select_tab)
		};
		Some(yard)
	}
}


#[derive(Clone, Debug)]
pub enum Action {
	SetNextDialog(u32),
	SetValue(i32),
	StringEdit(stringedit::Action),
	ShowTab(MainTab),
	OpenDialog,
	CloseDialog,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum MainTab {
	Dialog,
	FormList,
	SelectionList,
}

static TABS: &'static [(i32, &str)] = &[
	(1, "Dialog"),
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


