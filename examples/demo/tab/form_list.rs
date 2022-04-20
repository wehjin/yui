use std::iter::FromIterator;

use yui::app::Edge;
use yui::palette::{FillColor, StrokeColor};
use yui::palette::FillGrade::Plain;
use yui::prelude::*;
use yui::SenderLink;
use yui::yard::model::{ScrollModel, ScrollAction};
use yui::yard::ButtonState;

use crate::AppTab;

pub enum Action {
	StringEdit(stringedit::Action),
	ShowTab(usize),
	UpdateList(ScrollAction),
}

impl Spark for FormListDemo {
	type State = (StringEdit, ScrollModel);
	type Action = Action;
	type Report = usize;

	fn create<E: Edge>(&self, _create: &Create<Self::Action, Self::Report, E>) -> Self::State {
		let edit = StringEdit::empty(ValidIf::UnsignedInt);
		let list = ScrollModel::new(1930, vec![5, 5], 0);
		(edit, list)
	}


	fn flow(&self, action: Self::Action, flow: &impl Flow<Self::State, Self::Action, Self::Report>) -> AfterFlow<Self::State, Self::Report> {
		let (edit, list) = flow.state();
		match action {
			Action::StringEdit(edit_action) => {
				let edit = edit.edit(edit_action);
				AfterFlow::Revise((edit, list.clone()))
			}
			Action::ShowTab(index) => AfterFlow::Report(index),
			Action::UpdateList(action) => {
				if let Some(list) = list.update(action) {
					AfterFlow::Revise((edit.clone(), list))
				} else {
					AfterFlow::Ignore
				}
			}
		}
	}

	fn render(state: &Self::State, link: &SenderLink<Self::Action>) -> Option<ArcYard> {
		let (edit, list) = state;
		let mirror = yard::label(
			&String::from_iter(edit.chars.to_vec()),
			StrokeColor::BodyOnBackground,
			Cling::Left,
		);
		let fields = vec![
			{
				yard::textfield(
					1931,
					"Label".into(),
					edit.clone(),
					link.clone().map(Action::StringEdit),
				).confine_height(3, Cling::Center)
			},
			{
				let button = if edit.is_valid() {
					yard::button("Submit", ButtonState::enabled(link.clone().map(|_| Action::ShowTab(0))))
				} else {
					yard::button("Enter N", ButtonState::Disabled)
				};
				button.confine_height(3, Cling::Center)
			},
		];
		let list_link = link.to_sync().map(|action| Action::UpdateList(action));
		let list = yard::list(fields, list.clone(), list_link);
		let body = list
			.pack_top(3, mirror)
			.confine_width(50, Cling::Center)
			.pad(1)
			.before(yard::fill(FillColor::Background, Plain));
		let page = AppTab::FormList.page(body, Some(link.clone().map(Action::ShowTab)));
		Some(page)
	}
}

#[derive(Debug)]
pub struct FormListDemo {}


