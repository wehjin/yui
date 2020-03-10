use std::cell::RefCell;
use std::cmp::min;
use std::collections::HashMap;
use std::rc::Rc;

use crate::yui::Cling;

#[derive(Copy, Clone, Debug)]
pub struct Bounds {
	pub right: i32,
	pub bottom: i32,
	pub left: i32,
	pub top: i32,
	pub z: i32,
}

impl Bounds {
	pub fn new(width: i32, height: i32) -> Bounds {
		Bounds { right: width, bottom: height, left: 0, top: 0, z: 0 }
	}
	pub fn width(&self) -> i32 { self.right - self.left }
	pub fn height(&self) -> i32 { self.bottom - self.top }
	pub fn intersects(&self, row: i32, col: i32) -> bool {
		row >= self.top && row < self.bottom && col >= self.left && col < self.right
	}
	pub fn pad(&self, left_cols: i32, right_cols: i32, top_rows: i32, bottom_rows: i32) -> Bounds {
		Bounds {
			right: self.right - right_cols,
			bottom: self.bottom - bottom_rows,
			left: self.left + left_cols,
			top: self.top + top_rows,
			z: self.z,
		}
	}
	pub fn confine(&self, width: i32, height: i32, cling: Cling) -> Bounds {
		let (extra_width, extra_height) = (self.width() - width, self.height() - height);
		let top_extra = (cling.y() * extra_height as f32) as i32;
		let bottom_extra = extra_height - top_extra;
		let left_extra = (cling.x() * extra_width as f32) as i32;
		let right_extra = extra_width - left_extra;
		Bounds {
			right: self.right - right_extra,
			bottom: self.bottom - bottom_extra,
			left: self.left + left_extra,
			top: self.top + top_extra,
			z: self.z,
		}
	}
	pub fn with_z(&self, z: i32) -> Bounds {
		let mut new = self.clone();
		new.z = z;
		new
	}
	pub fn is_below(&self, other: &Bounds) -> bool {
		self.top >= other.bottom
	}
	pub fn is_above(&self, other: &Bounds) -> bool {
		self.bottom <= other.top
	}
	pub fn down_rank(&self, start: &Bounds) -> i32 {
		let distance = self.top - start.bottom;
		let overlap = start.x_overlap(self);
		distance - overlap
	}
	pub fn up_rank(&self, start: &Bounds) -> i32 {
		let distance = start.top - self.bottom;
		let overlap = start.x_overlap(self);
		distance - overlap
	}
	fn x_overlap(&self, target: &Bounds) -> i32 {
		if target.right < self.left {
			-(self.left - target.right)
		} else if target.right <= self.right {
			if target.left < self.left {
				target.right - self.left
			} else {
				target.width()
			}
		} else {
			if target.left < self.left {
				self.width()
			} else if target.left <= self.right {
				self.right - target.left
			} else {
				-(target.left - self.right)
			}
		}
	}
	pub fn split_from_top(&self, top_rows: i32) -> (Bounds, Bounds) {
		let middle = min(self.top + top_rows, self.bottom);
		let top = Bounds {
			right: self.right,
			bottom: middle,
			left: self.left,
			top: self.top,
			z: self.z,
		};
		let bottom = Bounds {
			right: self.right,
			bottom: self.bottom,
			left: self.left,
			top: middle,
			z: self.z,
		};
		(top, bottom)
	}
}

#[derive(Debug)]
pub struct BoundsHold {
	holdings: Vec<Bounds>,
	map: HashMap<i32, usize>,
}

impl BoundsHold {
	pub fn init(width: i32, height: i32) -> (usize, Rc<RefCell<BoundsHold>>) {
		let mut init_hold = BoundsHold::new();
		let init_index = init_hold.push_root(width, height);
		let hold_ref = Rc::new(RefCell::new(init_hold));
		(init_index, hold_ref)
	}

	pub fn new() -> BoundsHold {
		BoundsHold { holdings: Vec::new(), map: HashMap::new() }
	}

	pub fn bounds(&self, bounds_index: usize) -> Bounds {
		self.holdings.get(bounds_index).expect(&format!("No bounds at index {}", bounds_index)).to_owned()
	}

	pub fn push_root(&mut self, width: i32, height: i32) -> usize {
		self.push_bounds(&Bounds::new(width, height))
	}

	pub fn push_bounds(&mut self, bounds: &Bounds) -> usize {
		self.holdings.push(bounds.to_owned());
		self.holdings.len() - 1
	}

	pub fn yard_bounds(&self, id: i32) -> &Bounds {
		let bounds_index = self.map.get(&id).unwrap().to_owned();
		self.holdings.get(bounds_index).unwrap()
	}

	pub fn insert_yard_bounds(&mut self, id: i32, bounds_index: usize) {
		self.map.insert(id, bounds_index);
	}
}
