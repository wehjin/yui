extern crate log;
extern crate simplelog;
extern crate yui;

use std::error::Error;
use std::fs::File;

use log::LevelFilter;
use rand::random;
use simplelog::{Config, WriteLogger};

use yui::{AfterFlow, ArcYard, Before, Cling, Confine, console, Create, Flow, Pack, Padding, Sendable, SenderLink, Spark, yard};
use yui::app::Edge;
use yui::palette::{FillColor, FillGrade, StrokeColor};
use yui::yard::{Button2, Priority};

fn main() -> Result<(), Box<dyn Error>> {
	let log_file = File::create("counter.log")?;
	WriteLogger::init(LevelFilter::Info, Config::default(), log_file)?;
	log::info!("Counter");

	#[derive(Clone)]
	pub struct MainState {
		value: i32,
		buttons: Vec<Button2>,
	}
	impl MainState {
		pub fn increment(&self) -> Self {
			MainState { value: self.value + 1, ..self.clone() }
		}
		pub fn decrement(&self) -> Self {
			MainState { value: self.value - 1, ..self.clone() }
		}
		pub fn zero(&self) -> Self {
			MainState { value: 0, ..self.clone() }
		}
	}

	#[derive(Copy, Clone)]
	pub enum MainAction {
		Increment,
		Decrement,
		Zero,
		Done,
	}
	impl Sendable for MainAction {}

	#[derive(Copy, Clone)]
	pub struct Main;
	impl Spark for Main {
		type State = MainState;
		type Action = MainAction;
		type Report = ();

		fn create<E: Edge>(&self, ctx: &Create<Self::Action, Self::Report, E>) -> Self::State {
			let buttons = vec![
				Button2 { id: random(), label: "+".into(), priority: Priority::Default, submit: Some(ctx.link().map(|_| MainAction::Increment)) },
				Button2 { id: random(), label: "-".into(), priority: Priority::None, submit: Some(ctx.link().map(|_| MainAction::Decrement)) },
				Button2 { id: random(), label: "0".into(), priority: Priority::None, submit: Some(ctx.link().map(|_| MainAction::Zero)) },
				Button2 { id: random(), label: "X".into(), priority: Priority::None, submit: Some(ctx.link().map(|_| MainAction::Done)) },
			];
			MainState { value: 0, buttons }
		}

		fn flow(&self, action: Self::Action, ctx: &impl Flow<Self::State, Self::Action, Self::Report>) -> AfterFlow<Self::State, Self::Report> {
			match action {
				MainAction::Increment => AfterFlow::Revise(ctx.state().increment()),
				MainAction::Decrement => AfterFlow::Revise(ctx.state().decrement()),
				MainAction::Zero => AfterFlow::Revise(ctx.state().zero()),
				MainAction::Done => AfterFlow::Close(None),
			}
		}

		fn render(state: &Self::State, _link: &SenderLink<Self::Action>) -> Option<ArcYard> {
			let text = format!("{}", state.value);
			let label = yard::label(&text, StrokeColor::BodyOnBackground, Cling::Left);
			let buttons = state.buttons.iter().map(yard::button2)
				.rev()
				.map(|it| it.confine_width(5, Cling::Left))
				.fold(yard::empty(), |yard, button| yard.pack_left(7, button));
			let back = yard::fill(FillColor::Background, FillGrade::Plain);
			let front = yard::empty()
				.pack_top(3, buttons)
				.pack_top(3, label)
				.pad(20);
			let page = front.before(back);
			Some(page)
		}
	}
	console::run_spark(Main);
	Ok(())
}

