extern crate log;
extern crate simplelog;
extern crate yui;

use std::error::Error;
use std::fs::File;

use log::LevelFilter;
use rand::random;
use simplelog::{Config, WriteLogger};

use yui::{AfterFlow, ArcYard, Before, console, Create, Flow, Link, Pack, Padding, Sendable, SenderLink, Spark, yard};
use yui::app::Edge;
use yui::palette::FillColor::Background;
use yui::palette::FillGrade::Plain;
use yui::yard::{ButtonAction, ButtonModel, PressAction, PressModel, SubmitAffordance};
use yui::yard::model::{ScrollAction, ScrollModel};

fn main() -> Result<(), Box<dyn Error>> {
	WriteLogger::init(
		LevelFilter::Info,
		Config::default(),
		File::create("table.log").expect("log file"),
	).expect("result");
	log::info!("Table");

	let spark = Main();
	console::run_spark(spark);
	Ok(())
}

pub struct Main();

#[derive(Clone)]
pub enum MainAction {
	Close,
	PressButton,
	SubmitRow(usize),
	UpdateScroll(ScrollAction),
	UpdatePress(usize, PressAction),
}

impl Sendable for MainAction {}

impl Spark for Main {
	type State = (Vec<Vec<String>>, ScrollModel, ButtonModel, Vec<PressModel>);
	type Action = MainAction;
	type Report = ();

	fn create<E: Edge>(&self, ctx: &Create<Self::Action, Self::Report, E>) -> Self::State {
		let rows: Vec<Vec<String>> = (1..11).map(|it| vec![
			format!("{}", it),
			format!("SYM{}", it),
			format!("{}", 2 * it),
			format!("${:0.2}", 1.26 * it as f32),
		]).collect();
		let list = ScrollModel::new_count_height(random(), rows.len(), 3, 0);
		let button = ButtonModel {
			id: random(),
			label: "Close".into(),
			release_trigger: MainAction::Close.to_send(ctx.link()),
			affordance: SubmitAffordance::enabled(MainAction::PressButton.to_sync(ctx.link())),
		};
		let presses = rows.iter().enumerate()
			.map(|(index, _)| {
				let trigger = ctx.link().to_trigger(MainAction::SubmitRow(index));
				PressModel::new(random(), trigger)
			})
			.collect::<Vec<_>>();
		(rows, list, button, presses)
	}

	fn flow(&self, action: Self::Action, ctx: &impl Flow<Self::State, Self::Action, Self::Report>) -> AfterFlow<Self::State, Self::Report> {
		let (rows, scroll, button, presses) = ctx.state();
		match action {
			MainAction::Close => AfterFlow::Close(None),
			MainAction::PressButton => {
				let next_button = button.update(ButtonAction::Press);
				AfterFlow::Revise((rows.clone(), scroll.clone(), next_button, presses.clone()))
			}
			MainAction::SubmitRow(index) => {
				ctx.link().send(MainAction::UpdatePress(index, PressAction::Release));
				AfterFlow::Revise((rows.clone(), scroll.clone(), button.clone(), presses.clone()))
			}
			MainAction::UpdateScroll(action) => {
				if let Some(scroll) = scroll.update(action) {
					AfterFlow::Revise((rows.clone(), scroll, button.clone(), presses.clone()))
				} else {
					AfterFlow::Ignore
				}
			}
			MainAction::UpdatePress(index, action) => {
				let mut presses = presses.clone();
				let press = presses.remove(index).update(action);
				presses.insert(index, press);
				AfterFlow::Revise((rows.clone(), scroll.clone(), button.clone(), presses))
			}
		}
	}

	fn render(state: &Self::State, link: &SenderLink<Self::Action>) -> Option<ArcYard> {
		let (rows, list, button, presses) = state;
		let headers = vec![(6, "".into()), (40, "Symbol".into()), (20, "Shares".into()), (20, "Value".into())];
		let close_button = yard::button(button);
		let select_link = SenderLink::wrap_sink(|index| log::info!("Selected row index: {}", index));
		let list_link = link.to_sync().map(|action| MainAction::UpdateScroll(action));
		let page = yard::table(list.clone(), list_link, headers, rows.clone(), select_link, presses.clone())
			.pad(2)
			.pack_bottom(7, close_button.pad(2))
			.before(yard::fill(Background, Plain))
			;
		Some(page)
	}
}
