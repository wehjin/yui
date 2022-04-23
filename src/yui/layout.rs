use std::fmt::Debug;
use std::ops::Deref;
use std::rc::Rc;

use crate::core::bounds::Bounds;
use crate::yui::{Focus, FocusMotion, FocusMotionFuture, FocusType};

#[derive(Debug, Clone)]
pub struct ActiveFocus {
	pub focus: Option<Rc<Focus>>,
	pub peers: Vec<Rc<Focus>>,
	pub rear_z: i32,
}

impl Default for ActiveFocus {
	fn default() -> Self {
		ActiveFocus { focus: None, peers: Vec::new(), rear_z: 0 }
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
	pub fn expand_seam(&mut self, z: i32, depth: i32) {
		self.peers
			= self.to_foci()
			.into_iter()
			.map(|focus| {
				if z > focus.bounds.z {
					Rc::new(focus.expand_seam(z, depth))
				} else {
					focus
				}
			})
			.collect::<Vec<_>>();
		self.rear_z = if z > self.rear_z {
			self.rear_z - depth
		} else {
			self.rear_z
		};
		self.focus = None;
	}
	pub fn insert_seam(&mut self, from: &Self, z: i32, left: i32, top: i32) {
		let mut insert_foci
			= from.to_foci()
			.into_iter()
			.map(|focus| Rc::new(focus.shift_seam(z, left, top)))
			.collect::<Vec<_>>();
		let new_rear_z = if from.rear_z < i32::MAX {
			let insert_rear_z = from.rear_z + z;
			self.rear_z.min(insert_rear_z)
		} else {
			self.rear_z
		};
		self.peers = {
			let mut vec = self.to_foci();
			vec.append(&mut insert_foci);
			info!("FOCI AFTER INSERTION: {:?}", &vec);
			vec.into_iter().filter(|focus| focus.is_in_range(new_rear_z)).collect::<Vec<_>>()
		};
		self.rear_z = new_rear_z;
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

	fn next_focus(
		&self,
		include_bounds: impl Fn(&Bounds, &Bounds) -> bool,
		bounds_rank: impl Fn(&Bounds, &Bounds) -> i32,
	) -> ActiveFocus {
		if let Some(ref focus) = self.focus {
			let bounds = focus.bounds;
			let (mut targets, mut next_peers): (Vec<Rc<Focus>>, Vec<Rc<Focus>>)
				= self.peers.clone()
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
					rear_z: self.rear_z,
				}
			}
		} else {
			self.to_owned()
		}
	}
}
