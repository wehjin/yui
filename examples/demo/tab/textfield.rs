use std::iter::FromIterator;
use std::sync::Arc;

use stringedit::StringEdit;

use yui::{Before, Cling, Confine, Link, Padding, yard};
use yui::palette::{FillColor, StrokeColor};
use yui::yard::Yard;

use crate::{Action, tab_page};

pub fn render(edit: &StringEdit, link: &Link<Action>, select_tab: impl Fn(usize) + Sync + Send + 'static) -> Arc<dyn Yard + Sync + Send> {
	let link = link.clone();
	let trellis = yard::trellis(3, 1, vec![
		yard::label(
			&String::from_iter(edit.chars.to_vec()),
			StrokeColor::BodyOnBackground,
			Cling::Left,
		),
		yard::textfield(
			1932,
			"Label".into(),
			edit.clone(),
			move |new_edit| link.send(Action::StringEdit(new_edit)),
		),
	]);
	let content =
		trellis
			.confine(50, 7, Cling::Center)
			.pad(1)
			.before(yard::fill(FillColor::Background));
	tab_page(content, 1, select_tab)
}


