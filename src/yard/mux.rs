use crate::{ArcYard, Before, Cling, Confine, Pack, SyncLink};
use crate::palette::{FillColor, FillGrade};
use crate::yard;
use crate::yard::ButtonModel;
use crate::yard::model::{ScrollAction, ScrollModel};

pub fn mux(center: ArcYard, yards: Vec<ArcYard>, button: ButtonModel, scroll: ScrollModel, list_link: SyncLink<ScrollAction>) -> ArcYard {
	const SIDE_WIDTH: i32 = 40;
	let action_button = yard::button2(&button);
	let sidebar_fore = if yards.is_empty() {
		action_button.confine_height(3, Cling::Top)
	} else {
		let yards = yards.into_iter().enumerate().map(|(i, yard)| {
			if i == scroll.selected_index {
				yard.before(yard::fill(FillColor::Side, FillGrade::Select))
			} else {
				yard
			}
		}).collect::<Vec<_>>();
		yard::list(yards, scroll, list_link).pack_bottom(3, action_button)
	};
	let sidebar_back = yard::fill(FillColor::Side, FillGrade::Plain);
	let sidebar = sidebar_fore.before(sidebar_back);
	let yard = center.pack_left(SIDE_WIDTH, sidebar);
	yard
}
