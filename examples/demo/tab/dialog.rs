use yui::{AfterFlow, ArcYard, Before, Cling, Confine, Create, Flow, Link, Padding, Spark, yard};
use yui::palette::{FillColor, StrokeColor};

use crate::{Main, tab_page};

impl Spark for DialogDemo {
	type State = (u32, u32, Option<Link<Self::Report>>);
	type Action = Action;
	type Report = Report;

	fn render(state: &Self::State, link: &Link<Self::Action>) -> Option<ArcYard> {
		let (this_dialog, next_dialog, ref report_link) = *state;
		let gap_height = 1;
		let row_height = 3;
		let rows = vec![
			yard::label(&format!("{}", this_dialog), StrokeColor::BodyOnBackground, Cling::Center),
			{
				let link = link.clone();
				yard::button_enabled(&format!("Next {}", next_dialog), link.callback(|_| Action::Open))
			},
			{
				let link = link.clone();
				yard::button_enabled("Close", link.callback(|_| Action::Close))
			},
		];
		let min_trellis_height = rows.len() as i32 * (row_height + gap_height) - gap_height;
		let trellis = yard::trellis(row_height, gap_height, rows);
		let content = trellis.confine(32, min_trellis_height, Cling::Center)
			.pad(1)
			.before(yard::fill(FillColor::Background));

		let page = {
			let report_link = report_link.clone();
			tab_page(content, 0, move |n| if let Some(ref report_link) = report_link { report_link.send(Report::SelectedTab(n)) })
		};
		Some(page)
	}


	fn create(&self, create: &Create<Self::Action, Self::Report>) -> Self::State {
		(self.dialog, self.next_dialog, create.report_link().clone())
	}

	fn flow(&self, action: Self::Action, flow: &impl Flow<Self::State, Self::Action, Self::Report>) -> AfterFlow<Self::State, Self::Report> {
		match action {
			Action::Open => {
				let (_, next_dialog, _) = *flow.state();
				let link = flow.link().clone();
				flow.start_prequel(
					Main { dialog_id: next_dialog },
					move |next_dialog| link.send(Action::NextDialog(next_dialog)),
				);
				AfterFlow::Ignore
			}
			Action::Close => {
				let (_, next_dialog, _) = *flow.state();
				AfterFlow::Close(Some(Report::NextDialog(next_dialog)))
			}
			Action::NextDialog(next_dialog) => {
				let (dialog, _, ref reports) = *flow.state();
				let next = (dialog, next_dialog, reports.clone());
				AfterFlow::Revise(next)
			}
		}
	}
}

#[derive(Debug, Clone)]
pub struct DialogDemo {
	pub dialog: u32,
	pub next_dialog: u32,
}

pub enum Action {
	Open,
	Close,
	NextDialog(u32),
}

pub enum Report {
	SelectedTab(usize),
	NextDialog(u32),
}