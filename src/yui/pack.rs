use std::rc::Rc;

use crate::yui::{LayoutContext, Packing, RenderContext, Yard};
use crate::yui::layout::LayoutContextImpl;

impl Packing for Rc<dyn Yard> {
	fn pack_sides(self, size: i32) -> Rc<dyn Yard> {
		PackYard::new(size, self)
	}
}

struct PackYard {
	yard_id: i32,
	left_cols: i32,
	right_cols: i32,
	top_rows: i32,
	bottom_rows: i32,
	yard: Rc<dyn Yard>,
}

impl PackYard {
	fn new(size: i32, yard: Rc<dyn  Yard>) -> Rc<dyn Yard> {
		let cols = size * 2;
		let rows = size;
		Rc::new(PackYard {
			yard_id: rand::random(),
			left_cols: cols,
			right_cols: cols,
			top_rows: rows,
			bottom_rows: rows,
			yard,
		})
	}
}

impl Yard for PackYard {
	fn yard_id(&self) -> i32 { self.yard_id }

	fn layout(&self, ctx: &mut dyn LayoutContext) -> usize {
		let (index, bounds) = ctx.edge_bounds();
		let alt_bounds = bounds.pack(self.left_cols, self.right_cols, self.top_rows, self.bottom_rows);
		let alt_index = ctx.push_core_bounds(&alt_bounds);
		let mut alt_ctx = LayoutContextImpl::new(alt_index, ctx.bounds_hold().to_owned());
		self.yard.layout(&mut alt_ctx);
		// TODO Merge packed bounds into near/far.
		index
	}

	fn render(&self, ctx: &dyn RenderContext) {
		self.yard.render(ctx)
	}
}
