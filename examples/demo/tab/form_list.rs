use std::iter::FromIterator;
use std::sync::Arc;

use stringedit::StringEdit;

use yui::{Before, Cling, Confine, Link, Padding, yard};
use yui::palette::{FillColor, StrokeColor};
use yui::yard::Yard;

use crate::{Action, tab_page};

pub fn render(edit: &StringEdit, link: &Link<Action>, select_tab: impl Fn(usize) + Sync + Send + 'static) -> Arc<dyn Yard + Sync + Send> {
	let link = link.clone();
	let fields = vec![
		yard::label(
			&String::from_iter(edit.chars.to_vec()),
			StrokeColor::BodyOnBackground,
			Cling::Left,
		),
		yard::textfield(
			1931,
			"Label".into(),
			edit.clone(),
			move |new_edit| link.send(Action::StringEdit(new_edit)),
		).confine_height(3, Cling::Center),
	];
	let items = fields.into_iter().map(|it| (5u8, it)).collect();
	let list = yard::list(1930, 1, items);
	let content = list
		.confine_width(50, Cling::Center)
		.pad(1)
		.before(yard::fill(FillColor::Background));
	tab_page(content, 1, select_tab)
}


