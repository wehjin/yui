use yui::{AfterFlow, ArcYard, Cling, Create, Flow, Padding, SenderLink, Spark, yard};
use yui::palette::{FillColor, StrokeColor};
use yui::yard::{MuxButton, Pressable};

use crate::AppTab;

impl Spark for SelectorListDemo {
	type State = i32;
	type Action = Action;
	type Report = usize;

	fn create(&self, _create: &Create<Self::Action, Self::Report>) -> Self::State { 1 }

	fn flow(&self, action: Self::Action, _flow: &impl Flow<Self::State, Self::Action, Self::Report>) -> AfterFlow<Self::State, Self::Report> {
		match action {
			Action::SetValue(value) => AfterFlow::Revise(value),
			Action::ShowTab(index) => AfterFlow::Report(index),
		}
	}

	fn render(state: &Self::State, link: &SenderLink<Self::Action>) -> Option<ArcYard> {
		let value = *state;
		let mut items = Vec::new();
		for n in 1..11 {
			let quad_label = yard::quad_label(
				&format!("Item {}", n),
				"sub-title",
				&format!("{} Value", value),
				"2 sub-value",
				15,
				FillColor::Side,
			);
			let item = quad_label.pad(1).pressable(link.clone().map(move |_| Action::SetValue(n)));
			items.push((4, item));
		};
		let body = yard::mux(
			LIST_ID,
			yard::label(format!("Hello {}", value), StrokeColor::BodyOnBackground, Cling::Center),
			items,
			value as usize - 1,
			MuxButton("Add".into(), SenderLink::ignore()),
		);
		let page = AppTab::SelectorList.page(
			body,
			Some(link.clone().map(Action::ShowTab)),
		);
		Some(page)
	}
}

#[derive(Debug)]
pub struct SelectorListDemo {}

#[derive(Debug, Clone)]
pub enum Action {
	SetValue(i32),
	ShowTab(usize),
}


static LIST_ID: i32 = 22431;
