use yui::{AfterFlow, ArcYard, Cling, Confine, Create, Flow, Link, Padding, Spark, yard};
use yui::palette::FillColor;
use yui::yard::Pressable;

use crate::tab_page;

impl Spark for SelectorListDemo {
	type State = i32;
	type Action = Action;
	type Report = usize;

	fn yard(state: &Self::State, link: &Link<Self::Action>) -> Option<ArcYard> {
		let value = *state;
		let mut items = Vec::new();
		for n in 1..11 {
			let quad_label = yard::quad_label(
				&format!("Item {}", n),
				"sub-title",
				&format!("{} Value", value),
				"2 sub-value",
				15,
				FillColor::Background,
			);
			let item = quad_label.pad(1).pressable(link.callback(move |_| Action::SetValue(n)));
			items.push((4, item));
		};
		let body = yard::list(LIST_ID, value as usize - 1, items).confine_width(40, Cling::Center);
		let page = tab_page(body, 2, link.callback(|index| Action::ShowTab(index)));
		Some(page)
	}


	fn flow(_flow: &impl Flow<Self::State, Self::Action, Self::Report>, action: Self::Action) -> AfterFlow<Self::State, Self::Report> {
		match action {
			Action::SetValue(value) => AfterFlow::Revise(value),
			Action::ShowTab(index) => AfterFlow::Report(index),
		}
	}

	fn create(&self, _create: &Create<Self::Action, Self::Report>) -> Self::State { 1 }
}

#[derive(Debug)]
pub struct SelectorListDemo {}

#[derive(Debug, Clone)]
pub enum Action {
	SetValue(i32),
	ShowTab(usize),
}


static LIST_ID: i32 = 22431;
