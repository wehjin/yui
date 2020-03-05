use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub(crate) trait Yard {
	fn yard_id(&self) -> i32;
	fn layout(&self, ctx: &mut dyn LayoutContext) -> usize;
	fn render(&self, ctx: &dyn RenderContext);
}

pub(crate) trait LayoutContext {
	fn bounds_hold(&self) -> Rc<RefCell<BoundsHold>>;
	fn edge_bounds(&self) -> (usize, Bounds);
	fn push_core_bounds(&mut self, bounds: &Bounds) -> usize;
	fn set_yard_bounds(&mut self, yard_id: i32, bounds_index: usize);
}

pub(crate) trait RenderContext {
	fn spot(&self) -> (i32, i32);
	fn yard_bounds(&self, yard_id: i32) -> Bounds;
	fn set_fill(&self, row: i32, col: i32);
}


#[derive(Copy, Clone)]
pub struct Bounds {
	right: i32,
	bottom: i32,
	left: i32,
	top: i32,
	near: i32,
	far: i32,
}

impl Bounds {
	pub fn new(width: i32, height: i32) -> Bounds {
		Bounds { right: width, bottom: height, left: 0, top: 0, near: 0, far: 0 }
	}
	pub fn intersects(&self, row: i32, col: i32) -> bool {
		row >= self.top && row < self.bottom && col >= self.left && col < self.right
	}
	pub fn pack(&self, left_cols: i32, right_cols: i32, top_rows: i32, bottom_rows: i32) -> Bounds {
		Bounds {
			right: self.right - right_cols,
			bottom: self.bottom - bottom_rows,
			left: self.left + left_cols,
			top: self.top + top_rows,
			near: self.near,
			far: self.far,
		}
	}
}

pub(crate) struct BoundsHold {
	holdings: Vec<Bounds>,
	map: HashMap<i32, usize>,
}

impl BoundsHold {
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

	pub fn yard_bounds(&self, yard_id: i32) -> &Bounds {
		let bounds_index = self.map.get(&yard_id).unwrap().to_owned();
		self.holdings.get(bounds_index).unwrap()
	}

	pub fn insert_yard_bounds(&mut self, yard_id: i32, bounds_index: usize) {
		self.map.insert(yard_id, bounds_index);
	}
}
