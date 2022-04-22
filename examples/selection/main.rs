extern crate log;
extern crate simplelog;
extern crate yui;

use std::error::Error;
use std::fs::File;

use log::LevelFilter;
use simplelog::{Config, WriteLogger};

use yui::{AfterFlow, ArcYard, Before, Cling, Confine, console, Create, Flow, Pack, SenderLink, Spark, yard};
use yui::app::Edge;
use yui::palette::{FillColor, FillGrade, StrokeColor};
use yui::sparks::selection_editor::SelectionEditorSpark;
use yui::yard::{ButtonAction, ButtonModel};

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
	UpdateButton(ButtonAction),
	OpenSelector,
	Select(Option<(usize, usize)>),
}

impl Spark for Main {
	type State = (usize, ButtonModel);
	type Action = MainAction;
	type Report = ();

	fn create<E: Edge + Clone + Send + 'static>(&self, ctx: &Create<Self::Action, Self::Report, E>) -> Self::State {
		let value = 0usize;
		let release_trigger = ctx.link().to_trigger(MainAction::OpenSelector);
		let press_link = ctx.link().to_sync().map(|_| MainAction::UpdateButton(ButtonAction::Press));
		let button = ButtonModel::enabled("Open".into(), release_trigger, press_link);
		(value, button)
	}

	fn flow(&self, action: Self::Action, ctx: &impl Flow<Self::State, Self::Action, Self::Report>) -> AfterFlow<Self::State, Self::Report> {
		let (value, button) = ctx.state();
		match action {
			MainAction::UpdateButton(action) => {
				let button = button.update(action);
				AfterFlow::Revise((value.clone(), button))
			}
			MainAction::OpenSelector => {
				let spark = SelectionEditorSpark {
					selected: *value,
					choices: (0..10usize).into_iter().collect::<Vec<_>>(),
				};
				let report_link = ctx.link().map(|it| MainAction::Select(it));
				ctx.start_prequel(spark, report_link);
				let button = button.update(ButtonAction::Release);
				AfterFlow::Revise((value.clone(), button))
			}
			MainAction::Select(choice) => {
				if let Some((index, _value)) = choice {
					AfterFlow::Revise((index, button.clone()))
				} else {
					AfterFlow::Ignore
				}
			}
		}
	}

	fn render(state: &Self::State, _action_link: &SenderLink<Self::Action>) -> Option<ArcYard> {
		let (value, button) = state;
		let label = yard::label(format!("{}", value), StrokeColor::BodyOnBackground, Cling::Center);
		let button = yard::button(button).confine(10, 1, Cling::Center);
		let content = yard::empty().pack_top(3, button).pack_top(3, label).confine(40, 40, Cling::Center);
		let back = yard::fill(FillColor::Background, FillGrade::Plain);
		Some(content.before(back))
	}
}
