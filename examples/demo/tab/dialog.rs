use rand::random;

use yui::{AfterFlow, ArcYard, Before, Cling, Confine, Create, Flow, Padding, Sendable, SenderLink, Spark, yard};
use yui::app::Edge;
use yui::palette::{FillColor, StrokeColor};
use yui::palette::FillGrade::Plain;
use yui::yard::{ButtonModel, ButtonAction, SubmitAffordance};

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
	type State = (u32, u32, Option<SenderLink<Self::Report>>, DialogButtons);
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
		(self.dialog, self.next_dialog, create.report_link().clone(), buttons)
	}


	fn flow(&self, action: Self::Action, flow: &impl Flow<Self::State, Self::Action, Self::Report>) -> AfterFlow<Self::State, Self::Report> {
		match action {
			Action::PressClose => {
				let state = flow.state();
				AfterFlow::Revise((state.0, state.1, state.2.clone(), state.3.press_close()))
			}
			Action::Close => {
				let (_, next_dialog, _, _) = *flow.state();
				AfterFlow::Close(Some(Report::NextDialog(next_dialog)))
			}
			Action::PressOpen => {
				let state = flow.state();
				AfterFlow::Revise((state.0, state.1, state.2.clone(), state.3.press_open()))
			}
			Action::Open => {
				let (_, next_dialog, _, _) = *flow.state();
				let link = flow.link().clone();
				flow.start_prequel(
					Main { dialog_id: next_dialog },
					link.clone().map(|next_dialog| Action::NextDialog(next_dialog)),
				);
				AfterFlow::Ignore
			}
			Action::NextDialog(next_dialog) => {
				let (dialog, _, ref reports, ref buttons) = *flow.state();
				AfterFlow::Revise((dialog, next_dialog, reports.clone(), buttons.next_dialog(next_dialog)))
			}
		}
	}

	fn render(state: &Self::State, _link: &SenderLink<Self::Action>) -> Option<ArcYard> {
		let (this_dialog, _, ref report_link, ref buttons) = *state;
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

		let page = AppTab::Dialog.page(content, report_link.clone().map(|report_link| report_link.map(Report::SelectedTab)));
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
	SelectedTab(usize),
	NextDialog(u32),
}