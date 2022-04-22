use rand::random;

use yui::{AfterFlow, ArcYard, Before, Cling, Confine, Create, Flow, Padding, Sendable, SenderLink, Spark, yard};
use yui::app::Edge;
use yui::palette::{FillColor, StrokeColor};
use yui::palette::FillGrade::Plain;
use yui::yard::{ButtonAction, ButtonModel, SubmitAffordance};

use crate::{AppTab, Main};

fn next_label(n: u32) -> String { format!("Next {}", n) }

#[derive(Clone)]
pub struct DialogButtons {
	pub open: ButtonModel,
	pub close: ButtonModel,
}

impl DialogButtons {
	pub fn press_open(&self) -> Self { DialogButtons { open: self.open.update(ButtonAction::Press), close: self.close.clone() } }
	pub fn press_close(&self) -> Self { DialogButtons { open: self.open.clone(), close: self.close.update(ButtonAction::Press) } }
	pub fn next_dialog(&self, next_dialog: u32) -> Self {
		DialogButtons { open: self.open.set_label(&next_label(next_dialog)), close: self.close.clone() }
	}
}

#[derive(Clone)]
pub enum Action {
	PressClose,
	Close,
	PressOpen,
	Open,
	NextDialog(u32),
}

impl Spark for DialogDemo {
	type State = (u32, u32, DialogButtons);
	type Action = Action;
	type Report = Report;

	fn create<E: Edge>(&self, create: &Create<Self::Action, Self::Report, E>) -> Self::State {
		let buttons = DialogButtons {
			open: ButtonModel {
				id: random(),
				label: next_label(self.next_dialog),
				release_trigger: Action::Open.to_send(create.link()),
				affordance: SubmitAffordance::enabled(Action::PressOpen.to_sync(create.link())),
			},
			close: ButtonModel {
				id: random(),
				label: "Close".to_string(),
				release_trigger: Action::Close.to_send(create.link()),
				affordance: SubmitAffordance::enabled(Action::PressClose.to_sync(create.link())),
			},
		};
		(self.dialog, self.next_dialog, buttons)
	}


	fn flow(&self, action: Self::Action, flow: &impl Flow<Self::State, Self::Action, Self::Report>) -> AfterFlow<Self::State, Self::Report> {
		let (dialog, next_dialog, buttons) = flow.state();
		match action {
			Action::PressClose => {
				AfterFlow::Revise((dialog.clone(), next_dialog.clone(), buttons.press_close()))
			}
			Action::Close => {
				AfterFlow::Report(Report::ShouldCloseDialog(*next_dialog))
			}
			Action::PressOpen => {
				AfterFlow::Revise((dialog.clone(), next_dialog.clone(), buttons.press_open()))
			}
			Action::Open => {
				let link = flow.link().clone();
				flow.start_prequel(
					Main { dialog_id: next_dialog.clone() },
					link.clone().map(|next_dialog| Action::NextDialog(next_dialog)),
				);
				AfterFlow::Ignore
			}
			Action::NextDialog(next_dialog) => {
				AfterFlow::Revise((dialog.clone(), next_dialog.clone(), buttons.next_dialog(next_dialog)))
			}
		}
	}

	fn render(state: &Self::State, _link: &SenderLink<Self::Action>) -> Option<ArcYard> {
		let (this_dialog, _, ref buttons) = *state;
		let gap_height = 1;
		let row_height = 3;
		let rows = vec![
			yard::label(&format!("{}", this_dialog), StrokeColor::BodyOnBackground, Cling::Center),
			yard::button(&buttons.open),
			yard::button(&buttons.close),
		];
		let min_trellis_height = rows.len() as i32 * (row_height + gap_height) - gap_height;
		let trellis = yard::trellis(row_height, gap_height, Cling::Center, rows);
		let content = trellis.confine(32, min_trellis_height, Cling::Center)
			.pad(1)
			.before(yard::fill(FillColor::Background, Plain));

		let page = AppTab::Dialog.page(content);
		Some(page)
	}
}

#[derive(Debug, Clone)]
pub struct DialogDemo {
	pub dialog: u32,
	pub next_dialog: u32,
}

impl Sendable for Action {}

pub enum Report {
	ShouldCloseDialog(u32),
}