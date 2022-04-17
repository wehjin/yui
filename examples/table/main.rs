extern crate log;
extern crate simplelog;
extern crate yui;

use std::error::Error;
use std::fs::File;

use log::LevelFilter;
use simplelog::{Config, WriteLogger};

use yui::{AfterFlow, ArcYard, Before, console, Create, Flow, Pack, Padding, SenderLink, Spark, yard};
use yui::app::Edge;
use yui::palette::FillColor::Background;
use yui::palette::FillGrade::Plain;
use yui::yard::{ButtonState, Priority};

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

impl Spark for Main {
	type State = ();
	type Action = ();
	type Report = ();

	fn create<E: Edge>(&self, _ctx: &Create<Self::Action, Self::Report, E>) -> Self::State {
		()
	}

	fn flow(&self, _action: Self::Action, _ctx: &impl Flow<Self::State, Self::Action, Self::Report>) -> AfterFlow<Self::State, Self::Report> {
		AfterFlow::Close(None)
	}

	fn render(_state: &Self::State, link: &SenderLink<Self::Action>) -> Option<ArcYard> {
		let headers = vec![(6, "".into()), (40, "Symbol".into()), (20, "Shares".into()), (20, "Value".into())];
		let rows: Vec<Vec<String>> = (1..11).map(|it| vec![
			format!("{}", it),
			format!("SYM{}", it),
			format!("{}", 2 * it),
			format!("${:0.2}", 1.26 * it as f32),
		]).collect();
		let focus = 0;
		let close_button = yard::button(
			"Close".to_string(),
			ButtonState::Enabled(Priority::None, link.map(|_| ())),
		);
		let select_link = SenderLink::wrap_sink(|index| log::info!("Selected row index: {}", index));
		let page = yard::table(5000, focus, headers, rows, select_link)
			.pad(2)
			.pack_bottom(7, close_button.pad(2))
			.before(yard::fill(Background, Plain))
			;
		Some(page)
	}
}
