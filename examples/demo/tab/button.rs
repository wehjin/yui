use std::sync::Arc;

use yui::{Before, Cling, Confine, Link, Padding, yard};
use yui::palette::{FillColor, StrokeColor};
use yui::yard::Yard;

use crate::{Action, tab_page};

pub(crate) fn render(first_dialog: u32, next_dialog: u32, link: &Link<Action>, select_tab: impl Fn(usize) + Sync + Send + 'static) -> Arc<dyn Yard + Sync + Send> {
	let gap_height = 1;
	let row_height = 3;
	let rows = vec![
		yard::label(&format!("{}", first_dialog), StrokeColor::BodyOnBackground, Cling::Center),
		yard::button_enabled(&format!("Next {}", next_dialog), link.callback(|_| Action::OpenDialog)),
		yard::button_enabled("Close", link.callback(|_| Action::CloseDialog)),
	];
	let min_trellis_height = rows.len() as i32 * (row_height + gap_height) - gap_height;
	let trellis = yard::trellis(row_height, gap_height, rows);
	let content = trellis.confine(32, min_trellis_height, Cling::Center)
		.pad(1)
		.before(yard::fill(FillColor::Background));
	tab_page(content, 0, select_tab)
}
