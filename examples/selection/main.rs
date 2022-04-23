extern crate log;
extern crate simplelog;
extern crate yui;

use std::error::Error;
use std::fs::File;

use log::LevelFilter;
use simplelog::{Config, WriteLogger};

use yui::{AfterFlow, ArcYard, Before, Cling, Confine, console, Create, Flow, Pack, SenderLink, Spark, yard};
use yui::app::Edge;
use yui::palette::StrokeColor;
use yui::sparks::selection_editor::SelectionEditorSpark;
use yui::yard::{ButtonAction, ButtonModel, Priority};

fn main() -> Result<(), Box<dyn Error>> {
	WriteLogger::init(
		LevelFilter::Info,
		Config::default(),
		File::create("selection.log").expect("log file"),
	).expect("result");
	log::info!("Table");

	let spark = Main {};
	console::run_spark(spark);
	Ok(())
}

pub struct Main {}

#[derive(Clone)]
pub enum MainAction {
	UpdateOpen(ButtonAction),
	OpenSelector,
	Select(Option<(usize, usize)>),
	UpdateClose(ButtonAction),
	Close,
}

impl Spark for Main {
	type State = (usize, ButtonModel, ButtonModel);
	type Action = MainAction;
	type Report = ();

	fn create<E: Edge + Clone + Send + 'static>(&self, ctx: &Create<Self::Action, Self::Report, E>) -> Self::State {
		let value = 0usize;
		let open = ButtonModel::enabled(
			"Open".into(),
			ctx.link().to_trigger(MainAction::OpenSelector),
			ctx.link().to_sync().map(|_| MainAction::UpdateOpen(ButtonAction::Press)),
			Priority::Default,
		);
		let close = ButtonModel::enabled(
			"Close".into(),
			ctx.link().to_trigger(MainAction::Close),
			ctx.link().to_sync().map(|_| MainAction::UpdateClose(ButtonAction::Press)),
			Priority::None,
		);
		(value, open, close)
	}

	fn flow(&self, action: Self::Action, ctx: &impl Flow<Self::State, Self::Action, Self::Report>) -> AfterFlow<Self::State, Self::Report> {
		let (value, open, close) = ctx.state();
		match action {
			MainAction::UpdateOpen(action) => {
				let open = open.update(action);
				AfterFlow::Revise((value.clone(), open, close.clone()))
			}
			MainAction::OpenSelector => {
				let spark = SelectionEditorSpark {
					selected: *value,
					choices: (0..10usize).into_iter().collect::<Vec<_>>(),
				};
				let report_link = ctx.link().map(|it| MainAction::Select(it));
				ctx.start_prequel(spark, report_link);
				let open = open.update(ButtonAction::Release);
				AfterFlow::Revise((value.clone(), open, close.clone()))
			}
			MainAction::Select(choice) => {
				if let Some((index, _value)) = choice {
					AfterFlow::Revise((index, open.clone(), close.clone()))
				} else {
					AfterFlow::Ignore
				}
			}
			MainAction::UpdateClose(action) => {
				let close = close.update(action);
				AfterFlow::Revise((value.clone(), open.clone(), close))
			}
			MainAction::Close => {
				AfterFlow::Close(None)
			}
		}
	}

	fn render(state: &Self::State, _action_link: &SenderLink<Self::Action>) -> Option<ArcYard> {
		let (value, button, close) = state;
		let label = yard::label(format!("{}", value), StrokeColor::BodyOnBackground, Cling::Center);
		let button = yard::button(button).confine(10, 1, Cling::Center);
		let body = yard::empty().pack_top(3, button).pack_top(3, label).confine(40, 40, Cling::Center);
		let header = yard::button(close).confine_width(7, Cling::Left);
		let yard = body.pack_top(1, header).before(yard::fill_plain_background());
		Some(yard)
	}
}
