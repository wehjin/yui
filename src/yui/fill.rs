use std::rc::Rc;

use crate::yui::{LayoutContext, RenderContext, Yard};

pub fn fill_yard() -> Rc<dyn Yard> {
	FillYard::new()
}

struct FillYard {
	id: i32,
}

impl FillYard {
	fn new() -> Rc<dyn Yard> {
		let yard_id = rand::random();
		Rc::new(FillYard { id: yard_id })
	}
}

impl Yard for FillYard {
	fn id(&self) -> i32 {
		self.id
	}

	fn layout(&self, ctx: &mut dyn LayoutContext) -> usize {
		let (bounds_id, _bounds) = ctx.edge_bounds();
		ctx.set_yard_bounds(self.id(), bounds_id);
		bounds_id
	}

	fn render(&self, ctx: &dyn RenderContext) {
		let (row, col) = ctx.spot();
		let bounds = ctx.yard_bounds(self.id);
		if bounds.intersects(row, col) {
			ctx.set_fill(bounds.z)
		}
	}
}