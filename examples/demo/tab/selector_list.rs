use yui::{AfterFlow, ArcYard, Cling, Create, Flow, Padding, SenderLink, Spark, yard};
use yui::app::Edge;
use yui::palette::{FillColor, StrokeColor};
use yui::yard::{ButtonAction, ButtonModel};
use yui::yard::model::{ScrollAction, ScrollModel};

use crate::AppTab;

#[derive(Debug, Clone)]
pub enum Action {
	SetValue(i32),
	ShowTab(usize),
	UpdateScroll(ScrollAction),
	UpdateButton(ButtonAction),
}

impl Spark for SelectorListDemo {
	type State = (ScrollModel, ButtonModel);
	type Action = Action;
	type Report = usize;

	fn create<E: Edge>(&self, create: &Create<Self::Action, Self::Report, E>) -> Self::State {
		let scroll = ScrollModel::new_count_height(LIST_ID, 10, 4, 0);
		let release_trigger = create.link().to_trigger(Action::SetValue(1));
		let press_link = create.link().to_sync().map(|_| Action::UpdateButton(ButtonAction::Press));
		let button = ButtonModel::enabled("Add", release_trigger, press_link);
		(scroll, button)
	}

	fn flow(&self, action: Self::Action, flow: &impl Flow<Self::State, Self::Action, Self::Report>) -> AfterFlow<Self::State, Self::Report> {
		let (scroll, button) = flow.state();
		match action {
			Action::SetValue(value) => {
				let scroll = scroll.with_selected_index((value - 1) as usize);
				let button = button.update(ButtonAction::Release);
				AfterFlow::Revise((scroll, button))
			}
			Action::ShowTab(index) => {
				AfterFlow::Report(index)
			}
			Action::UpdateScroll(action) => {
				if let Some(scroll) = scroll.update(action) {
					AfterFlow::Revise((scroll, button.clone()))
				} else {
					AfterFlow::Ignore
				}
			}
			Action::UpdateButton(action) => {
				let button = button.update(action);
				AfterFlow::Revise((scroll.clone(), button))
			}
		}
	}

	fn render(state: &Self::State, link: &SenderLink<Self::Action>) -> Option<ArcYard> {
		let (scroll, button) = state;
		let count = scroll.item_count();
		let value = scroll.selected_index() + 1;
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

			let release_link = link.clone().map(move |_| Action::SetValue(n as i32));
			let item = yard::pressable(quad_label.pad(1), release_link);
			items.push(item);
		};
		let list_link = link.to_sync().map(|action| Action::UpdateScroll(action));
		let body = yard::mux(
			yard::label(format!("Hello {}", value), StrokeColor::BodyOnBackground, Cling::Center),
			items,
			button.clone(),
			scroll.clone(),
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
