use crate::{ArcYard, Before, Cling, Confine, Pack, SenderLink};
use crate::palette::{FillColor, FillGrade};
use crate::yard;
use crate::yard::ButtonState;

pub struct MuxButton(pub String, pub SenderLink<i32>);

pub fn mux(id: i32, center: ArcYard, sources: Vec<(u8, ArcYard)>, selected_index: usize, button: MuxButton) -> ArcYard {
	const SIDE_WIDTH: i32 = 40;
	let MuxButton(title, link) = button;
	let action_button = yard::button(title, ButtonState::enabled(link));
	let sidebar_fore = if sources.is_empty() {
		action_button.confine_height(3, Cling::Top)
	} else {
		let sources = sources.into_iter().enumerate().map(|(i, (size, yard))| {
			let adjusted_yard = if i == selected_index {
				yard.before(yard::fill(FillColor::Side, FillGrade::Select))
			} else { yard };
			(size, adjusted_yard)
		}).collect();
		yard::list(id, selected_index, sources).pack_bottom(3, action_button)
	};
	let sidebar_back = yard::fill(FillColor::Side, FillGrade::Plain);
	let sidebar = sidebar_fore.before(sidebar_back);
	let yard = center.pack_left(SIDE_WIDTH, sidebar);
	yard
}
