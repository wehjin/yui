use std::iter::FromIterator;

use yui::palette::{FillColor, StrokeColor};
use yui::palette::FillGrade::Plain;
use yui::prelude::*;
use yui::SenderLink;
use yui::yard::ButtonState;

use crate::AppTab;

impl Spark for FormListDemo {
	type State = StringEdit;
	type Action = Action;
	type Report = usize;

	fn create(&self, _create: &Create<Self::Action, Self::Report>) -> Self::State { StringEdit::empty(ValidIf::UnsignedInt) }


	fn flow(&self, action: Self::Action, flow: &impl Flow<Self::State, Self::Action, Self::Report>) -> AfterFlow<Self::State, Self::Report> {
		match action {
			Action::StringEdit(edit_action) => AfterFlow::Revise(flow.state().edit(edit_action)),
			Action::ShowTab(index) => AfterFlow::Report(index),
		}
	}

	fn render(edit: &Self::State, link: &SenderLink<Self::Action>) -> Option<ArcYard> {
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
		let items = fields.into_iter().map(|it| (5u8, it)).collect();
		let list = yard::list(1930, 0, items);
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

pub enum Action {
	StringEdit(stringedit::Action),
	ShowTab(usize),
}


