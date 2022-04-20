extern crate log;
extern crate simplelog;
extern crate yui;

use std::collections::HashMap;
use std::error::Error;
use std::fs::File;

use log::LevelFilter;
use rand::random;
use simplelog::{Config, WriteLogger};

use yui::{AfterFlow, ArcYard, Before, Cling, Confine, console, Create, Flow, Pack, Padding, Sendable, SenderLink, Spark, yard};
use yui::app::Edge;
use yui::palette::{FillColor, FillGrade, StrokeColor};
use yui::yard::{ButtonModel, ButtonAction, Priority, SubmitAffordance};

fn main() -> Result<(), Box<dyn Error>> {
	let log_file = File::create("counter.log")?;
	WriteLogger::init(LevelFilter::Info, Config::default(), log_file)?;
	log::info!("Counter");

	#[derive(Clone)]
	pub struct MainState {
		value: i32,
		buttons: Vec<ButtonModel>,
		index: HashMap<i32, usize>,
	}
	impl MainState {
		pub fn increment(&self) -> Self {
			MainState { value: self.value + 1, ..self.clone() }.release()
		}
		pub fn decrement(&self) -> Self {
			MainState { value: self.value - 1, ..self.clone() }.release()
		}
		pub fn zero(&self) -> Self {
			MainState { value: 0, ..self.clone() }.release()
		}
		pub fn press(&self, button_id: i32) -> Self {
			let position = self.index.get(&button_id).cloned().expect("Button position");
			let mut buttons = self.buttons.clone();
			let button = &buttons[position];
			buttons[position] = button.update(ButtonAction::Press);
			MainState { buttons, ..self.clone() }
		}
		fn release(&self) -> Self {
			let buttons = self.buttons.iter().map(|button| {
				button.update(ButtonAction::Release)
			}).collect();
			MainState { buttons, ..self.clone() }
		}
	}

	#[derive(Copy, Clone)]
	pub enum MainAction {
		Increment,
		Decrement,
		Zero,
		Done,
		Press(i32),
	}
	impl Sendable for MainAction {}

	#[derive(Copy, Clone)]
	pub struct Main;
	impl Spark for Main {
		type State = MainState;
		type Action = MainAction;
		type Report = ();

		fn create<E: Edge>(&self, ctx: &Create<Self::Action, Self::Report, E>) -> Self::State {
			let (plus, minus, zero, done) = (random(), random(), random(), random());
			let buttons = vec![
				ButtonModel {
					id: plus,
					label: "+".into(),
					affordance: SubmitAffordance::Enabled { priority: Priority::Default, press_link: MainAction::Press(plus).to_sync(ctx.link()) },
					release_trigger: MainAction::Increment.into_trigger_link(ctx.link()),
				},
				ButtonModel {
					id: minus,
					label: "-".into(),
					affordance: SubmitAffordance::Enabled { priority: Priority::None, press_link: MainAction::Press(minus).to_sync(ctx.link()) },
					release_trigger: MainAction::Decrement.into_trigger_link(ctx.link()),
				},
				ButtonModel {
					id: zero,
					label: "0".into(),
					affordance: SubmitAffordance::Enabled { priority: Priority::None, press_link: MainAction::Press(zero).to_sync(ctx.link()) },
					release_trigger: MainAction::Zero.into_trigger_link(ctx.link()),
				},
				ButtonModel {
					id: done,
					label: "X".into(),
					affordance: SubmitAffordance::Enabled { priority: Priority::None, press_link: MainAction::Press(done).to_sync(ctx.link()) },
					release_trigger: MainAction::Done.into_trigger_link(ctx.link()),
				},
			];
			let index = buttons.iter().enumerate().map(|(i, button)| (button.id, i)).collect::<HashMap<_, _>>();
			MainState { value: 0, buttons, index }
		}

		fn flow(&self, action: Self::Action, ctx: &impl Flow<Self::State, Self::Action, Self::Report>) -> AfterFlow<Self::State, Self::Report> {
			match action {
				MainAction::Press(button_id) => AfterFlow::Revise(ctx.state().press(button_id)),
				MainAction::Increment => AfterFlow::Revise(ctx.state().increment()),
				MainAction::Decrement => AfterFlow::Revise(ctx.state().decrement()),
				MainAction::Zero => AfterFlow::Revise(ctx.state().zero()),
				MainAction::Done => AfterFlow::Close(None),
			}
		}

		fn render(state: &Self::State, _link: &SenderLink<Self::Action>) -> Option<ArcYard> {
			let text = format!("{}", state.value);
			let label = yard::label(&text, StrokeColor::BodyOnBackground, Cling::Left);
			let buttons = state.buttons.iter().rev()
				.map(|button| yard::button2(button))
				.map(|yard| yard.confine_width(5, Cling::Left))
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

