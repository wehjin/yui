use std::cell::RefCell;
use std::rc::Rc;

use crate::yui::bounds::{Bounds, BoundsHold};
use crate::yui::Focus;

#[derive(Clone, Debug)]
pub struct LayoutContext {
	current_index: usize,
	bounds_hold: Rc<RefCell<BoundsHold>>,
	focus_vec: Rc<RefCell<Vec<Focus>>>,
}

impl LayoutContext {
	pub fn new(current_index: usize, bounds_hold: Rc<RefCell<BoundsHold>>) -> LayoutContext {
		LayoutContext {
			current_index,
			bounds_hold,
			focus_vec: Rc::new(RefCell::new(Vec::new())),
		}
	}

	pub fn focus_id(&self) -> i32 {
		match (*self.focus_vec).borrow().first() {
			Option::Some(focus) => focus.yard_id,
			_ => 0
		}
	}

	pub fn current_index(&self) -> usize {
		self.current_index
	}

	pub fn edge_bounds(&self) -> (usize, Bounds) {
		let bounds_index = self.current_index;
		let bounds = self.bounds(bounds_index);
		(bounds_index, bounds)
	}

	pub fn bounds(&self, index: usize) -> Bounds {
		(*self.bounds_hold).borrow().bounds(index)
	}

	pub fn push_bounds(&mut self, bounds: &Bounds) -> usize {
		(*self.bounds_hold).borrow_mut().push_bounds(bounds)
	}

	pub fn set_yard_bounds(&mut self, yard_id: i32, bounds_index: usize) {
		(*self.bounds_hold).borrow_mut().insert_yard_bounds(yard_id, bounds_index);
	}

	pub fn add_focus(&mut self, focus: Focus) {
		info!("FOCUS VEC: {:?}", self.focus_vec);
		info!("ADD FOCUS: {:?}", focus);
		(*self.focus_vec).borrow_mut().push(focus);
		info!("FOCUS VEC: {:?}", self.focus_vec);
	}

	pub fn with_index(&self, index: usize) -> LayoutContext {
		LayoutContext { current_index: index, ..self.to_owned() }
	}
}