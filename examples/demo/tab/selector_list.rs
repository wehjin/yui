use rand::random;

use yui::{AfterFlow, ArcYard, Cling, Create, Flow, Link, Padding, SenderLink, Spark, yard};
use yui::app::Edge;
use yui::palette::{FillColor, StrokeColor};
use yui::yard::{ButtonAction, ButtonModel, PressAction, PressModel};
use yui::yard::model::{ScrollAction, ScrollModel};

use crate::AppTab;

#[derive(Debug, Clone)]
pub enum Action {
	SetValue(i32),
	UpdateScroll(ScrollAction),
	UpdateButton(ButtonAction),
	UpdatePress(usize, PressAction),
}

impl Spark for SelectorListDemo {
	type State = (ScrollModel, ButtonModel, Vec<PressModel>);
	type Action = Action;
	type Report = usize;

	fn create<E: Edge>(&self, create: &Create<Self::Action, Self::Report, E>) -> Self::State {
		let count = 10;
		let scroll = ScrollModel::new_count_height(LIST_ID, count, 4, 0);
		let button = {
			let release_trigger = create.link().to_trigger(Action::SetValue(1));
			let press_link = create.link().to_sync().map(|_| Action::UpdateButton(ButtonAction::Press));
			ButtonModel::enabled("Add", release_trigger, press_link)
		};
		let presses = (0..count).into_iter()
			.map(|index| {
				PressModel::new(random(), create.link().to_trigger(Action::SetValue(index as i32 + 1)))
			})
			.collect::<Vec<_>>();
		(scroll, button, presses)
	}

	fn flow(&self, action: Self::Action, flow: &impl Flow<Self::State, Self::Action, Self::Report>) -> AfterFlow<Self::State, Self::Report> {
		let (scroll, button, presses) = flow.state();
		match action {
			Action::SetValue(value) => {
				let index = (value - 1) as usize;
				let scroll = scroll.with_selected_index(index);
				flow.link().send(Action::UpdateButton(ButtonAction::Release));
				flow.link().send(Action::UpdatePress(index, PressAction::Release));
				AfterFlow::Revise((scroll, button.clone(), presses.clone()))
			}
			Action::UpdateScroll(action) => {
				if let Some(scroll) = scroll.update(action) {
					AfterFlow::Revise((scroll, button.clone(), presses.clone()))
				} else {
					AfterFlow::Ignore
				}
			}
			Action::UpdateButton(action) => {
				let button = button.update(action);
				AfterFlow::Revise((scroll.clone(), button, presses.clone()))
			}
			Action::UpdatePress(index, action) => {
				let mut presses = presses.clone();
				let press = presses.remove(index).update(action);
				presses.insert(index, press);
				AfterFlow::Revise((scroll.clone(), button.clone(), presses))
			}
		}
	}

	fn render(state: &Self::State, link: &SenderLink<Self::Action>) -> Option<ArcYard> {
		let (scroll, button, presses) = state;
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
			let index = n - 1;
			let press_link = link.to_sync().map(move |_| Action::UpdatePress(index, PressAction::Press));
			let item = yard::pressable(quad_label.pad(1), &presses[index], press_link);
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
		let page = AppTab::SelectorList.page(body);
		Some(page)
	}
}

#[derive(Debug)]
pub struct SelectorListDemo {}


static LIST_ID: i32 = 22431;
