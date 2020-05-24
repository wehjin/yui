use std::sync::Arc;

use yui::{Before, Cling, Confine, Link, Padding, yard};
use yui::palette::FillColor;
use yui::yard::Yard;

use crate::{Action, tab_page};

pub(crate) fn render(link: &Link<Action>, select_tab: impl Fn(usize) + Sync + Send + 'static) -> Arc<dyn Yard + Sync + Send> {
	let trellis = yard::trellis(3, 2, vec![
		yard::button("Open  Dialog", link.callback(|_| Action::OpenDialog)),
		yard::button("Close", link.callback(|_| Action::CloseDialog)),
	]);
	let content = trellis.confine(32, 8, Cling::Center)
		.pad(1)
		.before(yard::fill(FillColor::Background));
	tab_page(content, 0, select_tab)
}
