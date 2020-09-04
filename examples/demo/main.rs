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

use yui::{app, Create, Flow, Link, Story};
use yui::{AfterFlow, ArcYard, Before, Cling, Pack, Padding, story, yard};
use yui::palette::{FillColor, StrokeColor};

use crate::tab::dialog::{DialogDemo, Report};
use crate::tab::form_list::FormListDemo;
use crate::tab::selector_list::SelectorListDemo;
use crate::tab::text_panel::TextDemo;

mod tab;

fn main() -> Result<(), Box<dyn Error>> {
	WriteLogger::init(LevelFilter::Info, Config::default(), File::create("yui.log").unwrap()).unwrap();
	info!("Demo");
	app::run(Demo { dialog_id: 1 }, None)
}

impl story::Spark for Demo {
	type State = State;
	type Action = MainTab;
	type Report = u32;

	fn render(state: &State, _link: &Link<Self::Action>) -> Option<ArcYard> {
		let State { main_tab, dialog_story, form_story, selector_story, text_story } = state;
		let yard = match main_tab {
			MainTab::Dialog => yard::publisher(dialog_story),
			MainTab::FormList => yard::publisher(form_story),
			MainTab::SelectorList => yard::publisher(selector_story),
			MainTab::Text => yard::publisher(text_story)
		};
		Some(yard)
	}

	fn create(&self, ctx: &Create<Self::Action, Self::Report>) -> Self::State {
		State {
			main_tab: MainTab::Dialog,
			dialog_story: {
				let report_link = ctx.report_link().clone();
				let action_link = ctx.link().clone();
				DialogDemo { dialog: self.dialog_id, next_dialog: self.dialog_id + 1 }.spark(
					ctx.edge().clone(),
					Some(Link::new(move |report| {
						match report {
							Report::SelectedTab(index) => action_link.send(tab_at_index(index)),
							Report::NextDialog(next_dialog) => if let Some(ref report_link) = report_link { report_link.send(next_dialog) },
						}
					})),
				)
			},
			form_story: FormListDemo {}.spark(
				ctx.edge().clone(),
				Some(Link::new(ctx.link().callback(|report| tab_at_index(report)))),
			),
			selector_story: SelectorListDemo {}.spark(
				ctx.edge().clone(),
				Some(Link::new(ctx.link().callback(|report| tab_at_index(report)))),
			),
			text_story: TextDemo {}.spark(
				ctx.edge().clone(),
				Some(Link::new(ctx.link().callback(|report| tab_at_index(report)))),
			),
		}
	}

	fn flow(&self, main_tab: Self::Action, flow: &impl Flow<Self::State, Self::Action, Self::Report>) -> AfterFlow<Self::State, Self::Report> {
		let next = flow.state().with_tab(main_tab);
		AfterFlow::Revise(next)
	}
}

pub struct Demo { dialog_id: u32 }

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
}


#[derive(Debug, Clone)]
pub enum MainTab {
	Dialog,
	FormList,
	SelectorList,
	Text,
}

static TABS: &'static [(i32, &str)] = &[
	(1, "Dialog"),
	(2, "Form List"),
	(3, "Selector List"),
	(4, "Text"),
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
		3 => MainTab::Text,
		_ => unimplemented!("No tab for index {}", index)
	}
}
