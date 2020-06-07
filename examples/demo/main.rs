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

use yui::{app, Create, Flow, Link, Story};
use yui::{AfterFlow, ArcYard, Before, Cling, Pack, Padding, story, yard};
use yui::palette::{FillColor, StrokeColor};
use yui::StringEdit;

use crate::tab::dialog::{DialogDemo, Report};
use crate::tab::selector_list::SelectorListDemo;

mod tab;

fn main() -> Result<(), Box<dyn Error>> {
	WriteLogger::init(LevelFilter::Info, Config::default(), File::create("yui.log").unwrap()).unwrap();
	info!("Demo");
	app::run(DemoSpark { dialog_id: 1 }, None)
}

#[derive(Debug, Clone)]
pub struct Demo {
	main_tab: MainTab,
	edit: StringEdit,
	dialog_story: Story<DialogDemo>,
	selector_story: Story<SelectorListDemo>,
}

impl Demo {
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
}

pub struct DemoSpark {
	dialog_id: u32,
}

impl story::Spark for DemoSpark {
	type State = Demo;
	type Action = Action;
	type Report = u32;

	fn yard(state: &Demo, link: &Link<Action>) -> Option<ArcYard> {
		let Demo { main_tab, edit, selector_story, dialog_story } = state;
		let select_tab = link.callback(|index| Action::ShowTab(tab_at_index(index)));
		let yard = match main_tab {
			MainTab::Dialog => yard::publisher(dialog_story),
			MainTab::FormList => tab::form_list::render(&edit, link, select_tab),
			MainTab::SelectorList => yard::publisher(selector_story),
		};
		Some(yard)
	}

	fn flow(flow: &impl Flow<Self::State, Self::Action, Self::Report>, action: Action) -> AfterFlow<Demo> {
		match action {
			Action::StringEdit(edit) => AfterFlow::Revise(flow.state().with_edit(edit)),
			Action::ShowTab(tab) => AfterFlow::Revise(flow.state().with_tab(tab)),
		}
	}

	fn create(&self, create: &Create<Self::Action, Self::Report>) -> Self::State {
		Demo {
			main_tab: MainTab::Dialog,
			edit: StringEdit::empty(Validity::UnsignedInt),
			selector_story: {
				let action_link = create.link().clone();
				SelectorListDemo {}.spark(
					create.edge().clone(),
					Some(Link::new(move |report| action_link.send(Action::ShowTab(tab_at_index(report))))),
				)
			},
			dialog_story: {
				let action_link = create.link().clone();
				let report_link = create.report_link().clone();
				DialogDemo { dialog: self.dialog_id, next_dialog: self.dialog_id + 1 }.spark(
					create.edge().clone(),
					Some(Link::new(move |report| {
						match report {
							Report::SelectedTab(index) => action_link.send(Action::ShowTab(tab_at_index(index))),
							Report::NextDialog(next_dialog) => if let Some(ref report_link) = report_link { report_link.send(next_dialog) },
						}
					})),
				)
			},
		}
	}
}


#[derive(Clone, Debug)]
pub enum Action {
	StringEdit(stringedit::Action),
	ShowTab(MainTab),
}

#[derive(Debug, Clone)]
pub enum MainTab {
	Dialog,
	FormList,
	SelectorList,
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
		.pack_top(3, yard::tabbar(TABS, active_tab_index, select_tab))
		.pack_top(3, app_bar())
}

fn tab_at_index(index: usize) -> MainTab {
	match index {
		0 => MainTab::Dialog,
		1 => MainTab::FormList,
		2 => MainTab::SelectorList,
		_ => unimplemented!("No tab for index {}", index)
	}
}
