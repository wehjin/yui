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
use yui::{console, Create, Flow, Link, SenderLink, Story};
use yui::{AfterFlow, ArcYard, Before, Cling, Padding, story, yard};
use yui::app::Edge;
use yui::palette::{FillColor, StrokeColor};
use yui::palette::FillGrade::Plain;

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
		State {
			main_tab: AppTab::from_index(0),
			dialog_story: {
				let report_link = ctx.report_link().clone();
				let action_link = ctx.link().clone();
				let edge = ctx.edge().clone();
				story::spark(
					DialogDemo { dialog: self.dialog_id, next_dialog: self.dialog_id + 1 },
					edge,
					Some(SenderLink::new_f(move |report| {
						match report {
							Report::SelectedTab(index) => action_link.send(SetTab(AppTab::from_index(index))),
							Report::NextDialog(next_dialog) => if let Some(ref report_link) = report_link { report_link.send(next_dialog) },
						}
					})),
				)
			},
			form_story: story::spark(FormListDemo {}, ctx.edge().clone(), Some(SenderLink::new_f(ctx.link().callback(select_tab)))),
			selector_story: story::spark(SelectorListDemo {}, ctx.edge().clone(), Some(SenderLink::new_f(ctx.link().callback(select_tab)))),
			text_story: story::spark(TextDemo {}, ctx.edge().clone(), Some(SenderLink::new_f(ctx.link().callback(select_tab)))),
			buttons_story: story::spark(ButtonDemo {}, ctx.edge().clone(), Some(SenderLink::new_f(ctx.link().callback(select_tab)))),
		}
	}

	fn flow(&self, action: Self::Action, flow: &impl Flow<Self::State, Self::Action, Self::Report>) -> AfterFlow<Self::State, Self::Report> {
		info!("{:?}", action);
		match action {
			SetTab(tab) => {
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
		let refresh_link = link.clone().map(|_| MainAction::Refresh);
		let yard = match state.main_tab {
			AppTab::Dialog => yard::publisher(&state.dialog_story, refresh_link.clone()),
			AppTab::FormList => yard::publisher(&state.form_story, refresh_link.clone()),
			AppTab::SelectorList => yard::publisher(&state.selector_story, refresh_link.clone()),
			AppTab::Text => yard::publisher(&state.text_story, refresh_link.clone()),
			AppTab::Buttons => yard::publisher(&state.buttons_story, refresh_link.clone()),
		};
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

#[derive(Debug, Clone)]
pub struct State {
	main_tab: AppTab,
	dialog_story: Story<DialogDemo>,
	form_story: Story<FormListDemo>,
	selector_story: Story<SelectorListDemo>,
	text_story: Story<TextDemo>,
	buttons_story: Story<ButtonDemo>,
}

fn app_bar() -> ArcYard {
	let tool_bar = yard::title("Components", StrokeColor::BodyOnPrimary, Cling::Custom { x: 0.0, y: 0.0 });
	let header_row = tool_bar.pad(1).before(yard::fill(FillColor::Primary, Plain));
	header_row
}

