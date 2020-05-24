use std::sync::Arc;

use yui::{Cling, Confine, Link, Padding, yard};
use yui::palette::FillColor;
use yui::yard::{Pressable, Yard};

use crate::{Action, tab_page};

pub fn render(value: i32, link: &Link<Action>, select_tab: impl Fn(usize) + Sync + Send + 'static) -> Arc<dyn Yard + Sync + Send> {
	let mut items = Vec::new();
	for n in 1..11 {
		let quad_label = yard::quad_label(
			&format!("Item {}", n),
			"sub-title",
			&format!("{} Value", value),
			"2 sub-value",
			15,
			FillColor::Background,
		);
		let link = link.clone();
		let item = quad_label.pad(1).pressable(move |_| {
			link.send(Action::SetValue(n))
		});
		items.push((4, item));
	};
	let content = yard::list(LIST_ID, value as usize - 1, items).confine_width(40, Cling::Center);
	tab_page(content, 2, select_tab)
}

static LIST_ID: i32 = 22431;
