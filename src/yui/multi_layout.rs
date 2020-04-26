use crate::yard::ArcYard;
use crate::yui::bounds::Bounds;
use crate::yui::layout::LayoutContext;

pub struct MultiLayout<'a> {
	ctx: &'a mut LayoutContext,
	start_index: usize,
	start_bounds: Bounds,
	near_z: i32,
	finished: bool,
}

impl<'a> MultiLayout<'a> {
	pub fn new(ctx: &'a mut LayoutContext) -> Self {
		let (index, bounds) = ctx.edge_bounds();
		MultiLayout { ctx, start_index: index, start_bounds: bounds, near_z: bounds.z, finished: false }
	}

	pub fn near_z(&self) -> i32 { self.near_z }

	pub fn layout(&mut self, yard: &ArcYard, bounds: &Bounds) {
		assert!(!self.finished);
		let out_index = if bounds == &self.start_bounds {
			yard.layout(self.ctx)
		} else {
			let in_index = self.ctx.push_bounds(bounds);
			let mut ctx = self.ctx.with_index(in_index);
			yard.layout(&mut ctx)
		};
		let out_z = self.ctx.bounds(out_index).z;
		if out_z < self.near_z {
			self.near_z = out_z;
		}
	}

	pub fn finish(&mut self) -> usize {
		assert!(!self.finished);
		self.finished = true;
		if self.near_z < self.start_bounds.z {
			let bounds = self.start_bounds.with_z(self.near_z);
			self.ctx.push_bounds(&bounds)
		} else {
			self.start_index
		}
	}
}
