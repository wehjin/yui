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

pub use app_tab::*;
use yui::{console, Create, Flow, Link, SenderLink};
use yui::{AfterFlow, ArcYard, Before, Cling, Padding, story, yard};
use yui::app::Edge;
use yui::palette::{FillColor, StrokeColor};
use yui::palette::FillGrade::Plain;
use yui::story_id::StoryId;

use crate::MainAction::SetTab;
use crate::tab::button_panel::ButtonDemo;
use crate::tab::dialog::{DialogDemo, Report};
use crate::tab::form_list::FormListDemo;
use crate::tab::selector_list::SelectorListDemo;
use crate::tab::text_panel::TextDemo;

mod tab;
mod app_tab;

fn select_tab(index: usize) -> MainAction {
	MainAction::SetTab(AppTab::from_index(index))
}

fn main() -> Result<(), Box<dyn Error>> {
	WriteLogger::init(
		LevelFilter::Info,
		Config::default(),
		File::create("yui.log").expect("create yui.log"),
	).expect("result");
	info!("Demo");
	let spark = Main { dialog_id: 1 };
	console::run_spark(spark);
	Ok(())
}

pub struct Main {
	dialog_id: u32,
}

#[derive(Debug, Clone)]
pub struct State {
	main_tab: AppTab,
	dialog_story: StoryId,
	form_story: StoryId,
	selector_story: StoryId,
	text_story: StoryId,
	buttons_story: StoryId,
}

#[derive(Debug)]
pub enum MainAction {
	SetTab(AppTab),
	Refresh,
}

impl story::Spark for Main {
	type State = State;
	type Action = MainAction;
	type Report = u32;

	fn create<E: Edge + Clone + Send + 'static>(&self, ctx: &Create<Self::Action, Self::Report, E>) -> Self::State where E: Clone
	{
		let edge = ctx.edge().clone().expect("edge in create");
		let dialog_reports = {
			let own_link = ctx.link().clone();
			let report_link = ctx.report_link().clone();
			Some(SenderLink::wrap_sink(move |report| match report {
				Report::SelectedTab(index) => own_link.send(SetTab(AppTab::from_index(index))),
				Report::NextDialog(next_dialog) => if let Some(ref report_link) = report_link { report_link.send(next_dialog) },
			}))
		};
		State {
			main_tab: AppTab::from_index(0),
			dialog_story: edge.sub_story(DialogDemo { dialog: self.dialog_id, next_dialog: self.dialog_id + 1 }, dialog_reports).story_id,
			form_story: edge.sub_story(FormListDemo {}, Some(SenderLink::wrap_sink(ctx.link().callback(select_tab)))).story_id,
			selector_story: edge.sub_story(SelectorListDemo {}, Some(SenderLink::wrap_sink(ctx.link().callback(select_tab)))).story_id,
			text_story: edge.sub_story(TextDemo {}, Some(SenderLink::wrap_sink(ctx.link().callback(select_tab)))).story_id,
			buttons_story: edge.sub_story(ButtonDemo {}, Some(SenderLink::wrap_sink(ctx.link().callback(select_tab)))).story_id,
		}
	}

	fn flow(&self, action: Self::Action, flow: &impl Flow<Self::State, Self::Action, Self::Report>) -> AfterFlow<Self::State, Self::Report> {
		info!("{:?}", action);
		match action {
			MainAction::SetTab(tab) => {
				let next = flow.state().with_tab(tab);
				AfterFlow::Revise(next)
			}
			MainAction::Refresh => {
				flow.redraw();
				AfterFlow::Ignore
			}
		}
	}

	fn render(state: &State, link: &SenderLink<Self::Action>) -> Option<ArcYard> {
		let sub_story_id = match state.main_tab {
			AppTab::Dialog => state.dialog_story,
			AppTab::FormList => state.form_story,
			AppTab::SelectorList => state.selector_story,
			AppTab::Text => state.text_story,
			AppTab::Buttons => state.buttons_story,
		};
		let content_yard = yard::story(sub_story_id);
		let select_tab = link.map(|index: usize| MainAction::SetTab(AppTab::from_index(index)));
		let yard = AppTab::main_page(content_yard, state.main_tab.index(), Some(select_tab));
		Some(yard)
	}
}

impl State {
	fn with_tab(&self, main_tab: AppTab) -> Self {
		let mut next = self.clone();
		next.main_tab = main_tab;
		next
	}
}

fn app_bar() -> ArcYard {
	let tool_bar = yard::title("Components", StrokeColor::BodyOnPrimary, Cling::Custom { x: 0.0, y: 0.0 });
	let header_row = tool_bar.pad(1).before(yard::fill(FillColor::Primary, Plain));
	header_row
}

