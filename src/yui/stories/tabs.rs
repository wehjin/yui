use crate::{AfterFlow, ArcYard, Create, Flow, SenderLink, Spark};
use crate::prelude::yard::Tab;

#[derive(Clone)]
pub struct TabsState {
	pub active_index: usize,
	pub tabs: Vec<(i32, String)>,
}

pub struct TabsSpark {
	pub tabs: Vec<(i32, String)>,
}

impl Spark for TabsSpark {
	type State = TabsState;
	type Action = usize;
	type Report = usize;

	fn create(&self, _ctx: &Create<Self::Action, Self::Report>) -> Self::State {
		let tabs = self.tabs.iter().map(|tab| (tab.uid(), tab.label().to_string())).collect();
		TabsState { active_index: 0, tabs }
	}

	fn flow(&self, action: Self::Action, _ctx: &impl Flow<Self::State, Self::Action, Self::Report>) -> AfterFlow<Self::State, Self::Report> {
		let select_index = action;
		AfterFlow::Report(select_index)
	}

	fn render(state: &Self::State, link: &SenderLink<Self::Action>) -> Option<ArcYard> {
		let tabs = &state.tabs;
		let yard = crate::yard::tabbar(tabs, state.active_index, link.clone());
		Some(yard)
	}
}

impl Tab for (i32, String) {
	fn uid(&self) -> i32 {
		let (uid, _) = self;
		*uid
	}

	fn label(&self) -> &str {
		let (_, label) = self;
		label
	}
}
