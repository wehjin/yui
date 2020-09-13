use std::sync::Arc;

use crate::{AfterFlow, ArcYard, Create, Fade, Flow, SyncLink, story, yard};
use crate::yard::YardPublisher;

pub(crate) struct PubStack {}

impl story::Spark for PubStack {
	type State = Vec<Arc<dyn YardPublisher>>;
	type Action = Action;
	type Report = ();

	fn render(vision: &Self::State, _link: &SyncLink<Self::Action>) -> Option<ArcYard> {
		if let Some(first_publisher) = vision.first() {
			let yard = vision[1..].iter().fold(
				yard::publisher(first_publisher),
				|rear_yard, publisher| {
					let fore_yard = yard::publisher(publisher);
					rear_yard.fade((10, 10), fore_yard)
				},
			);
			info!("New yard for YardStack");
			Some(yard)
		} else {
			None
		}
	}

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
		}
	}

	fn create(&self, _create: &Create<Self::Action, Self::Report>) -> Self::State { Vec::new() }
}

pub(crate) enum Action {
	Push(Arc<dyn YardPublisher>),
	Pop,
}
