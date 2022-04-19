extern crate log;
extern crate simplelog;
extern crate yui;

use std::error::Error;
use std::fs::File;

use log::LevelFilter;
use rand::random;
use simplelog::{Config, WriteLogger};

use yui::{AfterFlow, ArcYard, Before, console, Create, Flow, Pack, Padding, Sendable, SenderLink, Spark, yard};
use yui::app::Edge;
use yui::palette::FillColor::Background;
use yui::palette::FillGrade::Plain;
use yui::yard::{Button, ButtonAction, SubmitAffordance};

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
}

impl Sendable for MainAction {}

impl Spark for Main {
	type State = Button;
	type Action = MainAction;
	type Report = ();

	fn create<E: Edge>(&self, ctx: &Create<Self::Action, Self::Report, E>) -> Self::State {
		let button = Button {
			id: random(),
			label: "Close".into(),
			submit_link: MainAction::Close.to_send(ctx.link()),
			affordance: SubmitAffordance::enabled(MainAction::PressButton.to_sync(ctx.link())),
		};
		button
	}

	fn flow(&self, action: Self::Action, ctx: &impl Flow<Self::State, Self::Action, Self::Report>) -> AfterFlow<Self::State, Self::Report> {
		match action {
			MainAction::Close => AfterFlow::Close(None),
			MainAction::PressButton => AfterFlow::Revise(ctx.state().update(ButtonAction::Press)),
		}
	}

	fn render(state: &Self::State, _link: &SenderLink<Self::Action>) -> Option<ArcYard> {
		let headers = vec![(6, "".into()), (40, "Symbol".into()), (20, "Shares".into()), (20, "Value".into())];
		let rows: Vec<Vec<String>> = (1..11).map(|it| vec![
			format!("{}", it),
			format!("SYM{}", it),
			format!("{}", 2 * it),
			format!("${:0.2}", 1.26 * it as f32),
		]).collect();
		let focus = 0;
		let close_button = yard::button2(state);
		let select_link = SenderLink::wrap_sink(|index| log::info!("Selected row index: {}", index));
		let page = yard::table(5000, focus, headers, rows, select_link)
			.pad(2)
			.pack_bottom(7, close_button.pad(2))
			.before(yard::fill(Background, Plain))
			;
		Some(page)
	}
}
