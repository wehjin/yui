use std::iter::FromIterator;

use stringedit::{StringEdit, Validity};

use yui::{AfterFlow, ArcYard, Before, Cling, Confine, Create, Flow, Link, Pack, Padding, Spark, yard};
use yui::palette::{FillColor, StrokeColor};
use yui::yard::ButtonState;

use crate::tab_page;

impl Spark for FormListDemo {
	type State = StringEdit;
	type Action = Action;
	type Report = usize;

	fn create(&self, _create: &Create<Self::Action, Self::Report>) -> Self::State { StringEdit::empty(Validity::UnsignedInt) }


	fn flow(&self, action: Self::Action, flow: &impl Flow<Self::State, Self::Action, Self::Report>) -> AfterFlow<Self::State, Self::Report> {
		match action {
			Action::StringEdit(edit_action) => AfterFlow::Revise(flow.state().edit(edit_action)),
			Action::ShowTab(index) => AfterFlow::Report(index),
		}
	}

	fn render(edit: &Self::State, link: &Link<Self::Action>) -> Option<ArcYard> {
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
					link.callback(|new_edit| Action::StringEdit(new_edit)),
				).confine_height(3, Cling::Center)
			},
			{
				let button = if edit.is_valid() {
					yard::button("Submit", ButtonState::enabled(link.callback(|_| Action::ShowTab(0))))
				} else {
					yard::button("Enter N", ButtonState::Disabled)
				};
				button.confine_height(3, Cling::Center)
			}
		];
		let items = fields.into_iter().map(|it| (5u8, it)).collect();
		let list = yard::list(1930, 0, items);
		let body = list
			.pack_top(3, mirror)
			.confine_width(50, Cling::Center)
			.pad(1)
			.before(yard::fill(FillColor::Background));
		let page = tab_page(body, 1, link.callback(|index| Action::ShowTab(index)));
		Some(page)
	}
}

#[derive(Debug)]
pub struct FormListDemo {}

pub enum Action {
	StringEdit(stringedit::Action),
	ShowTab(usize),
}


