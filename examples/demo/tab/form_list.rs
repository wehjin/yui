use std::iter::FromIterator;

use yui::{Link, SenderLink};
use yui::app::Edge;
use yui::palette::{FillColor, StrokeColor};
use yui::palette::FillGrade::Plain;
use yui::prelude::*;
use yui::yard::{ButtonAction, ButtonModel};
use yui::yard::model::{ScrollAction, ScrollModel};

use crate::AppTab;
use crate::tab::form_list::Action::UpdateButton;

#[derive(Clone)]
pub enum Action {
	StringEdit(stringedit::Action),
	ShowTab(usize),
	UpdateScroll(ScrollAction),
	UpdateButton(ButtonAction),
}

const DISABLED_TEXT: &str = "Enter N";

impl Spark for FormListDemo {
	type State = (StringEdit, ScrollModel, ButtonModel);
	type Action = Action;
	type Report = usize;

	fn create<E: Edge>(&self, create: &Create<Self::Action, Self::Report, E>) -> Self::State {
		let edit = StringEdit::empty(ValidIf::UnsignedInt);
		let list = ScrollModel::new(1930, vec![5, 5], 0);
		let button = ButtonModel::disabled(DISABLED_TEXT, create.link().to_trigger(Action::ShowTab(0)));
		(edit, list, button)
	}


	fn flow(&self, action: Self::Action, flow: &impl Flow<Self::State, Self::Action, Self::Report>) -> AfterFlow<Self::State, Self::Report> {
		let (edit, list, button) = flow.state();
		match action {
			Action::StringEdit(edit_action) => {
				let edit = edit.edit(edit_action);
				let button = if edit.is_valid() {
					button.enable("Submit", flow.link().to_sync().map(|_| UpdateButton(ButtonAction::Press)))
				} else {
					button.disable(DISABLED_TEXT)
				};
				AfterFlow::Revise((edit, list.clone(), button))
			}
			Action::ShowTab(index) => {
				flow.link().send(Action::UpdateButton(ButtonAction::Release));
				AfterFlow::Report(index)
			}
			Action::UpdateScroll(action) => {
				if let Some(list) = list.update(action) {
					AfterFlow::Revise((edit.clone(), list, button.clone()))
				} else {
					AfterFlow::Ignore
				}
			}
			Action::UpdateButton(action) => {
				let button = button.update(action);
				AfterFlow::Revise((edit.clone(), list.clone(), button))
			}
		}
	}

	fn render(state: &Self::State, link: &SenderLink<Self::Action>) -> Option<ArcYard> {
		let (edit, list, button) = state;
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
				let button = yard::button(button);
				button.confine_height(3, Cling::Center)
			},
		];
		let list_link = link.to_sync().map(|action| Action::UpdateScroll(action));
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


