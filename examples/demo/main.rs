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

use yui::{Create, Flow, SyncLink, Story, Link};
use yui::{AfterFlow, ArcYard, Before, Cling, Pack, Padding, story, yard};
use yui::palette::{FillColor, StrokeColor};

use crate::tab::button_panel::ButtonDemo;
use crate::tab::dialog::{DialogDemo, Report};
use crate::tab::form_list::FormListDemo;
use crate::tab::selector_list::SelectorListDemo;
use crate::tab::text_panel::TextDemo;

mod tab;

static TABS: &'static [(i32, &str)] = &[
	(1, "Dialog"),
	(2, "Form List"),
	(3, "Selector List"),
	(4, "Text"),
	(5, "Buttons"),
];

#[derive(Debug, Clone)]
pub enum MainTab {
	Dialog,
	FormList,
	SelectorList,
	Text,
	Buttons,
}

impl From<usize> for MainTab {
	fn from(index: usize) -> Self {
		match index {
			0 => MainTab::Dialog,
			1 => MainTab::FormList,
			2 => MainTab::SelectorList,
			3 => MainTab::Text,
			4 => MainTab::Buttons,
			_ => unimplemented!("No tab for index {}", index)
		}
	}
}

fn main() -> Result<(), Box<dyn Error>> {
	WriteLogger::init(LevelFilter::Info, Config::default(), File::create("yui.log").unwrap()).unwrap();
	info!("Demo");
	yui::main(Main { dialog_id: 1 })
}

pub struct Main { dialog_id: u32 }

impl story::Spark for Main {
	type State = State;
	type Action = MainTab;
	type Report = u32;

	fn create(&self, ctx: &Create<Self::Action, Self::Report>) -> Self::State {
		State {
			main_tab: MainTab::Dialog,
			dialog_story: {
				let report_link = ctx.report_link().clone();
				let action_link = ctx.link().clone();
				story::spark(
					DialogDemo { dialog: self.dialog_id, next_dialog: self.dialog_id + 1 },
					ctx.edge().clone(),
					Some(SyncLink::new(move |report| {
						match report {
							Report::SelectedTab(index) => action_link.send(MainTab::from(index)),
							Report::NextDialog(next_dialog) => if let Some(ref report_link) = report_link { report_link.send(next_dialog) },
						}
					})),
				)
			},
			form_story: story::spark(FormListDemo {}, ctx.edge().clone(), Some(SyncLink::new(ctx.link().callback(MainTab::from)))),
			selector_story: story::spark(SelectorListDemo {}, ctx.edge().clone(), Some(SyncLink::new(ctx.link().callback(MainTab::from)))),
			text_story: story::spark(TextDemo {}, ctx.edge().clone(), Some(SyncLink::new(ctx.link().callback(MainTab::from)))),
			buttons_story: story::spark(ButtonDemo {}, ctx.edge().clone(), Some(SyncLink::new(ctx.link().callback(MainTab::from)))),
		}
	}

	fn flow(&self, main_tab: Self::Action, flow: &impl Flow<Self::State, Self::Action, Self::Report>) -> AfterFlow<Self::State, Self::Report> {
		info!("{:?}", main_tab);
		let next = flow.state().with_tab(main_tab);
		AfterFlow::Revise(next)
	}

	fn render(state: &State, _link: &SyncLink<Self::Action>) -> Option<ArcYard> {
		let yard = match state.main_tab {
			MainTab::Dialog => yard::publisher(&state.dialog_story),
			MainTab::FormList => yard::publisher(&state.form_story),
			MainTab::SelectorList => yard::publisher(&state.selector_story),
			MainTab::Text => yard::publisher(&state.text_story),
			MainTab::Buttons => yard::publisher(&state.buttons_story),
		};
		Some(yard)
	}
}

impl State {
	fn with_tab(&self, main_tab: MainTab) -> Self {
		let mut next = self.clone();
		next.main_tab = main_tab;
		next
	}
}

#[derive(Debug, Clone)]
pub struct State {
	main_tab: MainTab,
	dialog_story: Story<DialogDemo>,
	form_story: Story<FormListDemo>,
	selector_story: Story<SelectorListDemo>,
	text_story: Story<TextDemo>,
	buttons_story: Story<ButtonDemo>,
}

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
