use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;

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
				FocusType::Edit(on_motion) | FocusType::CompositeSubmit(on_motion) => on_motion.deref()(motion),
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
	focus_max: i32,
	refresh_fn: Arc<dyn Fn() + Sync + Send>,
}

fn pick_priority_focus(mut candidates: Vec<Rc<Focus>>) -> (Option<Rc<Focus>>, Vec<Rc<Focus>>) {
	let (_, max_priority_index) = candidates.iter().enumerate().fold(
		(0, None),
		|(max_priority, max_priority_index), (focus_index, focus)| {
			if max_priority_index.is_none() || focus.priority > max_priority {
				(focus.priority, Some(focus_index))
			} else {
				(max_priority, max_priority_index)
			}
		},
	);
	match max_priority_index {
		None => (None, candidates),
		Some(index) => {
			let focus = candidates.remove(index);
			(Some(focus), candidates)
		}
	}
}

impl LayoutContext {
	pub fn refresh_fn(&self) -> Arc<dyn Fn() + Sync + Send> { self.refresh_fn.clone() }
	pub fn trapped_focus(&self) -> Option<Rc<Focus>> {
		self.focus_vec.borrow().last().map(|it| it.clone())
	}
	pub fn pop_active_focus(&mut self, past_active: &ActiveFocus) -> ActiveFocus {
		let available_foci = self.all_focus_in_range();
		let (focus, peers) =
			if let ActiveFocus { focus: Some(past_focus), .. } = past_active {
				let (mut continuity_foci, new_foci): (Vec<Rc<Focus>>, Vec<Rc<Focus>>) = available_foci.into_iter().partition(|it| it.yard_id == past_focus.yard_id);
				if continuity_foci.is_empty() {
					pick_priority_focus(new_foci)
				} else {
					let focus = continuity_foci.remove(0);
					(Some(focus), new_foci)
				}
			} else {
				pick_priority_focus(available_foci)
			};
		ActiveFocus { focus, peers }
	}

	pub fn all_focus_in_range(&self) -> Vec<Rc<Focus>> {
		let all_focus = (*self.focus_vec).borrow().clone();
		all_focus.into_iter().filter(|it| it.is_in_range(self.focus_max)).collect()
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

	pub fn set_yard_bounds(&mut self, yard_id: i32, bounds_index: usize) -> usize {
		(*self.bounds_hold).borrow_mut().insert_yard_bounds(yard_id, bounds_index);
		bounds_index
	}

	pub fn add_focus(&mut self, focus: Focus) {
		(*self.focus_vec).borrow_mut().push(Rc::new(focus));
	}

	pub fn set_focus_max(&mut self, focus_max: i32) {
		self.focus_max = focus_max
	}

	pub fn trap_foci(&self) -> Self {
		let mut new_context = self.to_owned();
		new_context.focus_vec = Rc::new(RefCell::new(Vec::new()));
		new_context
	}

	pub fn with_index(&self, index: usize) -> Self {
		LayoutContext { current_index: index, ..self.to_owned() }
	}

	pub fn new(current_index: usize, bounds_hold: Rc<RefCell<BoundsHold>>, refresh_fn: impl Fn() + Sync + Send + 'static) -> Self {
		LayoutContext {
			current_index,
			bounds_hold,
			focus_vec: Rc::new(RefCell::new(Vec::new())),
			focus_max: i32::max_value(),
			refresh_fn: Arc::new(refresh_fn),
		}
	}
}