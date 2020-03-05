use std::collections::HashMap;

pub(crate) trait Yard {
	fn yard_id(&self) -> i32;
	fn layout(&self, ctx: &mut dyn LayoutContext) -> usize;
	fn render(&self, ctx: &dyn RenderContext);
}

pub(crate) trait LayoutContext {
	fn edge_bounds(&self) -> (usize, &Bounds);
	fn set_yard_bounds(&mut self, yard_id: i32, bounds_id: usize);
	fn release_hold(self) -> BoundsHold;
}

pub(crate) trait RenderContext {
	fn spot(&self) -> (i32, i32);
	fn yard_bounds(&self, yard_id: i32) -> &Bounds;
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
	pub fn zero() -> Bounds {
		Bounds { right: 0, bottom: 0, left: 0, top: 0, near: 0, far: 0 }
	}
	pub fn new(width: i32, height: i32) -> Bounds {
		Bounds { right: width, bottom: height, left: 0, top: 0, near: 0, far: 0 }
	}
	pub fn intersects(&self, row: i32, col: i32) -> bool {
		row >= self.top && row < self.bottom && col >= self.left && col < self.right
	}
	pub fn pack(&self, rows: i32, cols: i32) -> Bounds {
		Bounds {
			right: self.right - cols,
			bottom: self.bottom - rows,
			left: self.left + cols,
			top: self.top + rows,
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

	pub fn get_bounds(&self, bounds_index: usize) -> &Bounds {
		self.holdings.get(bounds_index).expect(&format!("No bounds at index {}", bounds_index))
	}

	pub fn push_bounds(&mut self, width: i32, height: i32) -> usize {
		self.holdings.push(Bounds::new(width, height));
		self.holdings.len() - 1
	}

	pub fn get_yard_bounds(&self, yard_id: i32) -> &Bounds {
		let bounds_index = self.map.get(&yard_id).unwrap().to_owned();
		self.holdings.get(bounds_index).unwrap()
	}

	pub fn insert_yard_bounds(&mut self, yard_id: i32, bounds_index: usize) {
		self.map.insert(yard_id, bounds_index);
	}
}
