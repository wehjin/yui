use std::cell::RefCell;
use std::rc::Rc;

use crate::yui::bounds::{Bounds, BoundsHold};
use crate::yui::LayoutContext;

pub struct LayoutContextImpl {
	current_index: usize,
	bounds_hold: Rc<RefCell<BoundsHold>>,
}

impl LayoutContextImpl {
	pub fn new(current_index: usize, bounds_hold: Rc<RefCell<BoundsHold>>) -> LayoutContextImpl {
		LayoutContextImpl { current_index, bounds_hold }
	}
}

impl LayoutContext for LayoutContextImpl {
	fn edge_bounds(&self) -> (usize, Bounds) {
		let bounds_index = self.current_index;
		let bounds = self.bounds_hold.borrow().bounds(bounds_index);
		(bounds_index, bounds)
	}

	fn bounds(&self, index: usize) -> Bounds {
		self.bounds_hold.borrow().bounds(index)
	}

	fn push_bounds(&mut self, bounds: &Bounds) -> usize {
		self.bounds_hold.borrow_mut().push_bounds(bounds)
	}

	fn set_yard_bounds(&mut self, yard_id: i32, bounds_index: usize) {
		self.bounds_hold.borrow_mut().insert_yard_bounds(yard_id, bounds_index);
	}

	fn bounds_hold(&self) -> Rc<RefCell<BoundsHold>> {
		self.bounds_hold.clone()
	}
}