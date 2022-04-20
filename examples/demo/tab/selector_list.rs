use yui::{AfterFlow, ArcYard, Cling, Create, Flow, Padding, SenderLink, Spark, yard};
use yui::app::Edge;
use yui::palette::{FillColor, StrokeColor};
use yui::yard::{MuxButton, Pressable};
use yui::yard::model::{ScrollModel, ScrollAction};

use crate::AppTab;

#[derive(Debug, Clone)]
pub enum Action {
	SetValue(i32),
	ShowTab(usize),
	ToList(ScrollAction),
}

impl Spark for SelectorListDemo {
	type State = ScrollModel;
	type Action = Action;
	type Report = usize;

	fn create<E: Edge>(&self, _create: &Create<Self::Action, Self::Report, E>) -> Self::State {
		let list = ScrollModel::new_count_height(LIST_ID, 10, 4, 0);
		list
	}

	fn flow(&self, action: Self::Action, flow: &impl Flow<Self::State, Self::Action, Self::Report>) -> AfterFlow<Self::State, Self::Report> {
		let list = flow.state();
		match action {
			Action::SetValue(value) => {
				let list = list.with_selected_index((value - 1) as usize);
				AfterFlow::Revise(list)
			}
			Action::ShowTab(index) => {
				AfterFlow::Report(index)
			}
			Action::ToList(action) => {
				if let Some(list) = list.update(action) {
					AfterFlow::Revise(list)
				} else {
					AfterFlow::Ignore
				}
			}
		}
	}

	fn render(state: &Self::State, link: &SenderLink<Self::Action>) -> Option<ArcYard> {
		let list = state;
		let count = list.item_count();
		let value = list.selected_index() + 1;
		let mut items = Vec::new();
		for n in 1..(count + 1) {
			let quad_label = yard::quad_label(
				&format!("Item {}", n),
				"sub-title",
				&format!("{} Value", value),
				"2 sub-value",
				15,
				FillColor::Side,
			);
			let item = quad_label.pad(1).pressable(link.clone().map(move |_| Action::SetValue(n as i32)));
			items.push(item);
		};
		let list_link = link.to_sync().map(|action| Action::ToList(action));
		let body = yard::mux(
			yard::label(format!("Hello {}", value), StrokeColor::BodyOnBackground, Cling::Center),
			items,
			MuxButton("Add".into(), SenderLink::ignore()),
			list.clone(),
			list_link,
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


static LIST_ID: i32 = 22431;
