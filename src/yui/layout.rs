use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

use crate::yui::{Focus, FocusMotion, FocusMotionFuture, FocusType};
use crate::yui::bounds::{Bounds, BoundsHold};

#[derive(Clone)]
pub struct ActiveFocus {
	focus: Option<Rc<Focus>>,
	peers: Vec<Rc<Focus>>,
}

impl Default for ActiveFocus {
	fn default() -> Self {
		ActiveFocus { focus: None, peers: Vec::new() }
	}
}

impl ActiveFocus {
	pub fn focus_id(&self) -> i32 {
		match self.focus {
			Some(ref focus) => focus.yard_id,
			None => 0
		}
	}

	pub fn insert_space(&self, refresh: impl Fn() + Send + 'static) {
		if let Some(ref focus) = self.focus {
			focus.insert_space(refresh);
		}
	}

	pub fn insert_char(&self, char: char, refresh: impl Fn() + Send + 'static) {
		if let Some(ref focus) = self.focus {
			focus.insert_char(char, refresh);
		}
	}

	pub fn move_up(&self) -> ActiveFocus {
		if self.send_motion(FocusMotion::Up) == FocusMotionFuture::Default {
			self.next_focus(
				|bounds, origin| bounds.is_above(origin),
				|bounds, origin| bounds.up_rank(origin),
			)
		} else {
			self.to_owned()
		}
	}

	fn send_motion(&self, motion: FocusMotion) -> FocusMotionFuture {
		if let Some(ref focus) = self.focus {
			match &focus.focus_type {
				FocusType::Submit => FocusMotionFuture::Default,
				FocusType::Edit(on_motion) => on_motion.deref()(motion),
			}
		} else {
			FocusMotionFuture::Default
		}
	}

	pub fn move_down(&self) -> ActiveFocus {
		if self.send_motion(FocusMotion::Down) == FocusMotionFuture::Default {
			self.next_focus(
				|bounds, origin| bounds.is_below(origin),
				|bounds, origin| bounds.down_rank(origin),
			)
		} else {
			self.to_owned()
		}
	}

	pub fn move_left(&self) -> ActiveFocus {
		if self.send_motion(FocusMotion::Left) == FocusMotionFuture::Default {
			self.next_focus(
				|bounds, origin| bounds.is_left_of(origin),
				|bounds, origin| bounds.left_rank(origin),
			)
		} else {
			self.to_owned()
		}
	}

	pub fn move_right(&self) -> ActiveFocus {
		if self.send_motion(FocusMotion::Right) == FocusMotionFuture::Default {
			self.next_focus(
				|bounds, origin| bounds.is_right_of(origin),
				|bounds, origin| bounds.right_rank(origin),
			)
		} else {
			self.to_owned()
		}
	}

	fn next_focus(&self,
	              include_bounds: impl Fn(&Bounds, &Bounds) -> bool,
	              bounds_rank: impl Fn(&Bounds, &Bounds) -> i32,
	) -> ActiveFocus {
		if let Some(ref focus) = self.focus {
			let bounds = focus.bounds;
			let (mut targets, mut next_peers): (Vec<Rc<Focus>>, Vec<Rc<Focus>>) =
				self.peers.clone()
					.into_iter()
					.partition(|it| include_bounds(&it.bounds, &bounds));
			targets.sort_by_key(|it| bounds_rank(&it.bounds, &bounds));
			if targets.is_empty() {
				self.to_owned()
			} else {
				let next_focus = targets.remove(0);
				next_peers.append(&mut targets);
				next_peers.push(focus.clone());
				ActiveFocus {
					focus: Some(next_focus),
					peers: next_peers,
				}
			}
		} else {
			self.to_owned()
		}
	}
}

#[derive(Clone)]
pub struct LayoutContext {
	current_index: usize,
	bounds_hold: Rc<RefCell<BoundsHold>>,
	focus_vec: Rc<RefCell<Vec<Rc<Focus>>>>,
}

impl LayoutContext {
	pub fn new(current_index: usize, bounds_hold: Rc<RefCell<BoundsHold>>) -> LayoutContext {
		LayoutContext {
			current_index,
			bounds_hold,
			focus_vec: Rc::new(RefCell::new(Vec::new())),
		}
	}

	pub fn pop_active_focus(&mut self, active: &ActiveFocus) -> ActiveFocus {
		let mut all_focus = (*self.focus_vec).borrow().clone();
		let next_active = if let ActiveFocus { focus: Some(focus), .. } = active {
			let (mut found, peers): (Vec<Rc<Focus>>, Vec<Rc<Focus>>) = all_focus.into_iter().partition(|it| it.yard_id == focus.yard_id);
			let next_focus = if found.is_empty() {
				None
			} else {
				Some(found.remove(0))
			};
			ActiveFocus { focus: next_focus, peers }
		} else {
			if all_focus.is_empty() {
				ActiveFocus { focus: None, peers: all_focus }
			} else {
				let focus = Some(all_focus.remove(0));
				ActiveFocus { focus, peers: all_focus }
			}
		};
		next_active
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
		(*self.focus_vec).borrow_mut().push(Rc::new(focus));
	}

	pub fn with_index(&self, index: usize) -> LayoutContext {
		LayoutContext { current_index: index, ..self.to_owned() }
	}
}