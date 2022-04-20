use crate::{ArcYard, Before, Cling, Confine, Pack, SenderLink, SyncLink};
use crate::palette::{FillColor, FillGrade};
use crate::yard;
use crate::yard::model::{ScrollModel, ScrollAction};
use crate::yard::ButtonState;

pub struct MuxButton(pub String, pub SenderLink<i32>);

pub fn mux(center: ArcYard, yards: Vec<ArcYard>, button: MuxButton, list_art: ScrollModel, list_link: SyncLink<ScrollAction>) -> ArcYard {
	const SIDE_WIDTH: i32 = 40;
	let MuxButton(title, link) = button;
	let action_button = yard::button(title, ButtonState::enabled(link));
	let sidebar_fore = if yards.is_empty() {
		action_button.confine_height(3, Cling::Top)
	} else {
		let yards = yards.into_iter().enumerate().map(|(i, yard)| {
			if i == list_art.selected_index {
				yard.before(yard::fill(FillColor::Side, FillGrade::Select))
			} else {
				yard
			}
		}).collect::<Vec<_>>();
		yard::list(yards, list_art, list_link).pack_bottom(3, action_button)
	};
	let sidebar_back = yard::fill(FillColor::Side, FillGrade::Plain);
	let sidebar = sidebar_fore.before(sidebar_back);
	let yard = center.pack_left(SIDE_WIDTH, sidebar);
	yard
}
