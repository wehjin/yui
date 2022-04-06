use std::cell::RefCell;
use std::rc::Rc;

use crate::{ArcYard, Bounds};
use crate::bounds::BoundsHold;
use crate::spot::spot_table::SpotTable;

pub fn run(yard: &ArcYard, max_x: i32, max_y: i32, bounds_hold: Rc<RefCell<BoundsHold>>, focus_id: i32) -> SpotTable {
	let bounds_hold = bounds_hold.borrow();
	info!("({},{}) Bounds hold: {:?}", max_x, max_y, bounds_hold);
	let mut draw_pad = SpotTable::new(max_y, max_x);
	let mut tasks: Vec<(ArcYard, i32)> = vec![(yard.clone(), focus_id)];
	loop {
		let empty_bounds = Bounds::new(0, 0);
		if let Some((yard, focus_id)) = tasks.pop() {
			let bounds = bounds_hold.yard_bounds(yard.id()).unwrap_or(&empty_bounds);
			let more = yard.render(bounds, focus_id, &mut draw_pad);
			if let Some(more) = more {
				more.iter().for_each(|(yard, alt)| {
					tasks.insert(0, (yard.clone(), alt.unwrap_or(focus_id)))
				})
			}
		} else {
			break;
		}
	}
	draw_pad
}
