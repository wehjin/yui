
use std::ops::Deref;
use std::rc::Rc;


use crate::yui::{Focus, FocusMotion, FocusMotionFuture, FocusType};
use crate::yui::bounds::{Bounds};

#[derive(Clone)]
pub struct ActiveFocus {
	pub focus: Option<Rc<Focus>>,
	pub peers: Vec<Rc<Focus>>,
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
