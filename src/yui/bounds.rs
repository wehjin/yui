use std::cell::RefCell;
use std::cmp::min;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Copy, Clone)]
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
	pub fn with_z(&self, z: i32) -> Bounds {
		let mut new = self.clone();
		new.z = z;
		new
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
