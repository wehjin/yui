extern crate log;
extern crate simplelog;
extern crate yui;

use std::error::Error;
use std::fs::File;

use log::{info, LevelFilter};
use rand::random;
use simplelog::{Config, WriteLogger};

use yui::{AfterFlow, ArcYard, Before, console, Create, Flow, Pack, Padding, Sendable, SenderLink, Spark, yard};
use yui::app::Edge;
use yui::palette::FillColor::Background;
use yui::palette::FillGrade::Plain;
use yui::yard::{Button, ButtonAction, SubmitAffordance};
use yui::yard::model::{ScrollModel, ScrollAction};

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
	ToListArt(ScrollAction),
}

impl Sendable for MainAction {}

impl Spark for Main {
	type State = (Vec<Vec<String>>, ScrollModel, Button);
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
		let button = Button {
			id: random(),
			label: "Close".into(),
			submit_link: MainAction::Close.to_send(ctx.link()),
			affordance: SubmitAffordance::enabled(MainAction::PressButton.to_sync(ctx.link())),
		};
		(rows, list, button)
	}

	fn flow(&self, action: Self::Action, ctx: &impl Flow<Self::State, Self::Action, Self::Report>) -> AfterFlow<Self::State, Self::Report> {
		match action {
			MainAction::Close => AfterFlow::Close(None),
			MainAction::PressButton => {
				let state = ctx.state();
				let next_button = state.2.update(ButtonAction::Press);
				AfterFlow::Revise((state.0.clone(), state.1.clone(), next_button))
			}
			MainAction::ToListArt(action) => {
				let (rows, list, button) = ctx.state();
				if let Some(list) = list.update(action) {
					info!("Nexux: {:?}", &list.nexus);
					AfterFlow::Revise((rows.clone(), list, button.clone()))
				} else {
					AfterFlow::Ignore
				}
			}
		}
	}

	fn render(state: &Self::State, link: &SenderLink<Self::Action>) -> Option<ArcYard> {
		let (rows, list, button) = state;
		let headers = vec![(6, "".into()), (40, "Symbol".into()), (20, "Shares".into()), (20, "Value".into())];
		let close_button = yard::button2(button);
		let select_link = SenderLink::wrap_sink(|index| log::info!("Selected row index: {}", index));
		let list_link = link.to_sync().map(|action| MainAction::ToListArt(action));
		let page = yard::table(list.clone(), list_link, headers, rows.clone(), select_link)
			.pad(2)
			.pack_bottom(7, close_button.pad(2))
			.before(yard::fill(Background, Plain))
			;
		Some(page)
	}
}
