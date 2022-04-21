use std::fmt;

use rand::random;

use crate::{AfterFlow, ArcYard, Before, Cling, Confine, Create, FillColor, FillGrade, Flow, Link, Pack, Padding, SenderLink, Spark};
use crate::app::Edge;
use crate::palette::StrokeColor;
use crate::sparks::selection_editor::SelectionAction::{UpdateButton, UpdatePress};
use crate::yard::{ButtonAction, ButtonModel, PressAction, PressModel};
use crate::yard::model::{ScrollAction, ScrollModel};
use crate::yui::prelude::yard;

pub struct SelectionEditorSpark<T> {
	pub selected: usize,
	pub choices: Vec<T>,
}

#[derive(Debug, Clone)]
pub enum SelectionAction {
	SelectIndex(usize),
	Close,
	UpdatePress(usize, PressAction),
	UpdateButton(ButtonAction),
	UpdateScroll(ScrollAction),
}

impl<T: Clone + Send + fmt::Display> Spark for SelectionEditorSpark<T> {
	type State = (Vec<T>, Vec<PressModel>, ScrollModel, ButtonModel);
	type Action = SelectionAction;
	type Report = Option<(usize, T)>;

	fn create<E: Edge>(&self, ctx: &Create<Self::Action, Self::Report, E>) -> Self::State {
		let choices = self.choices.clone();
		let presses = choices.iter().enumerate()
			.map(|(i, _)| {
				let release_trigger = ctx.link().to_trigger(SelectionAction::SelectIndex(i));
				PressModel::new(random(), release_trigger)
			})
			.collect::<Vec<_>>();
		let scroll = ScrollModel::new_count_height(random(), choices.len(), 3, self.selected);
		let button = {
			let release_trigger = ctx.link().to_trigger(SelectionAction::Close);
			let press_link = ctx.link().to_sync().map(|_| SelectionAction::UpdateButton(ButtonAction::Press));
			ButtonModel::enabled("Close", release_trigger, press_link)
		};
		(choices, presses, scroll, button)
	}

	fn flow(&self, action: Self::Action, ctx: &impl Flow<Self::State, Self::Action, Self::Report>) -> AfterFlow<Self::State, Self::Report> {
		let (choices, presses, scroll, button) = ctx.state();
		match action {
			SelectionAction::SelectIndex(index) => {
				ctx.link().send(UpdatePress(index, PressAction::Release));
				let scroll = scroll.with_selected_index(index);
				AfterFlow::Revise((choices.clone(), presses.clone(), scroll, button.clone()))
			}
			SelectionAction::Close => {
				ctx.link().send(UpdateButton(ButtonAction::Release));
				ctx.end_prequel();
				AfterFlow::Report(None)
			}
			SelectionAction::UpdatePress(index, action) => {
				let mut presses = presses.clone();
				let press = presses.remove(index).update(action);
				presses.insert(index, press);
				AfterFlow::Revise((choices.clone(), presses, scroll.clone(), button.clone()))
			}
			SelectionAction::UpdateButton(action) => {
				let button = button.update(action);
				AfterFlow::Revise((choices.clone(), presses.clone(), scroll.clone(), button))
			}
			SelectionAction::UpdateScroll(action) => {
				if let Some(scroll) = scroll.update(action) {
					AfterFlow::Revise((choices.clone(), presses.clone(), scroll, button.clone()))
				} else {
					AfterFlow::Ignore
				}
			}
		}
	}

	fn render(state: &Self::State, link: &SenderLink<Self::Action>) -> Option<ArcYard> {
		let (choices, presses, scroll, button) = state;
		let content = if choices.is_empty() {
			yard::label("Empty", StrokeColor::CommentOnBackground, Cling::Center)
		} else {
			let selected_index = scroll.selected_index();
			let yards = choices.iter().enumerate()
				.map(|(index, value)| {
					let (text, color) = if selected_index == index {
						(format!("{}", value).to_uppercase(), StrokeColor::BodyOnBackground)
					} else {
						(format!("{}", value), StrokeColor::EnabledOnBackground)
					};
					let yard = yard::label(text, color, Cling::Center);
					let press = &presses[index];
					let press_link = link.to_sync().map(move |_| SelectionAction::SelectIndex(index));
					yard::pressable(yard, press, press_link)
				})
				.collect::<Vec<_>>();

			let link = link.clone();
			let scroll_link = link.to_sync().map(|action| SelectionAction::UpdateScroll(action));
			yard::list(yards, scroll.clone(), scroll_link)
		};
		let close_button = yard::button(button)
			.confine_width(9, Cling::Center);
		let background = yard::fill(FillColor::Background, FillGrade::Plain);
		let yard = content
			.pack_bottom(1, close_button)
			.pad(1)
			.before(background);
		Some(yard)
	}
}

