use std::sync::mpsc::SyncSender;

use crate::{AfterFlow, ArcYard, Create, Fade, Flow, SenderLink, story, yard};
use crate::app::Edge;
use crate::yard::YardControlMsg;

pub(crate) struct PubStack {}

impl story::Spark for PubStack {
	type State = Vec<SyncSender<YardControlMsg>>;
	type Action = Action;
	type Report = ();

	fn create<E: Edge>(&self, _create: &Create<Self::Action, Self::Report, E>) -> Self::State { Vec::new() }

	fn flow(&self, action: Self::Action, flow: &impl Flow<Self::State, Self::Action, Self::Report>) -> AfterFlow<Self::State, Self::Report> {
		match action {
			Action::Pop => {
				if flow.state().len() == 1 {
					flow.report(());
					AfterFlow::Ignore
				} else {
					let mut next_state = flow.state().to_vec();
					next_state.pop();
					AfterFlow::Revise(next_state)
				}
			}
			Action::Push(front) => {
				let mut next_state = flow.state().to_vec();
				next_state.push(front);
				AfterFlow::Revise(next_state)
			}
			Action::Refresh => {
				flow.redraw();
				AfterFlow::Ignore
			}
		}
	}

	fn render(vision: &Self::State, link: &SenderLink<Self::Action>) -> Option<ArcYard> {
		let refresh = link.clone().map(|_| Action::Refresh);
		if let Some(first_publisher) = vision.first() {
			let publisher = yard::publisher(first_publisher, refresh.clone());
			let yard = vision[1..].iter().fold(
				publisher,
				|rear_yard, publisher| {
					let fore_yard = yard::publisher(publisher, refresh.clone());
					rear_yard.fade((4, 4), fore_yard)
				},
			);
			info!("New yard for YardStack");
			Some(yard)
		} else {
			None
		}
	}
}

pub(crate) enum Action {
	Push(SyncSender<YardControlMsg>),
	Pop,
	Refresh,
}
