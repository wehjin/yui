use std::fmt::Debug;
use std::ops::Deref;
use std::rc::Rc;

use crate::yui::{Focus, FocusMotion, FocusMotionFuture, FocusType};
use crate::yui::bounds::Bounds;

#[derive(Debug, Clone)]
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
	pub fn to_foci(&self) -> Vec<Rc<Focus>> {
		let mut all = self.peers.clone();
		if let Some(focus) = &self.focus {
			all.push(focus.clone())
		}
		all
	}
	pub fn insert_seam(&mut self, from: &Self, z: i32, left: i32, top: i32) {
		for peer in &from.peers {
			let focus = peer.shift_seam(z, left, top);
			self.peers.push(Rc::new(focus));
		}
		if let Some(focus) = &from.focus {
			let new = focus.shift_seam(z, left, top);
			self.peers.push(Rc::new(new));
		}
	}
	pub fn expand_seam(&mut self, z: i32, depth: i32) {
		let mut new_peers = self.peers.iter().map(|it| Rc::new(it.expand_seam(z, depth)))
			.collect::<Vec<_>>();
		if let Some(focus) = &self.focus {
			let new_focus = focus.expand_seam(z, depth);
			new_peers.push(Rc::new(new_focus));
		}
		self.peers = new_peers;
		self.focus = None;
	}

	pub fn focus_id(&self) -> i32 {
		match self.focus {
			Some(ref focus) => focus.yard_id,
			None => 0
		}
	}
	pub fn nearest_z(&self) -> i32 {
		let near = if let Some(focus) = &self.focus {
			focus.bounds.z.min(0)
		} else { 0 };
		self.peers.iter().fold(near, |near, more| {
			near.min(more.bounds.z)
		})
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
