extern crate log;
extern crate simplelog;
extern crate yui;

use std::error::Error;
use std::fs::File;

use log::LevelFilter;
use simplelog::{Config, WriteLogger};

use yui::{AfterFlow, ArcYard, Before, Cling, console, Create, Flow, SenderLink, Spark, yard};
use yui::app::Edge;
use yui::palette::{FillColor, FillGrade, StrokeColor};

fn main() -> Result<(), Box<dyn Error>> {
	let log_file = File::create("hello.log")?;
	WriteLogger::init(LevelFilter::Info, Config::default(), log_file)?;
	log::info!("Hello!");

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

		fn render(_state: &Self::State, _link: &SenderLink<Self::Action>) -> Option<ArcYard> {
			let label = yard::label("Hello, world!", StrokeColor::BodyOnBackground, Cling::Center);
			let back = yard::fill(FillColor::Background, FillGrade::Plain);
			let page = label.before(back);
			Some(page)
		}
	}
	let spark = Main();
	console::run_spark(spark);
	Ok(())
}

