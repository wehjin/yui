use std::rc::Rc;

use crate::yui::{LayoutContext, Padding, RenderContext, Yard};
use crate::yui::layout::LayoutContextImpl;

impl Padding for Rc<dyn Yard> {
	fn pad_sides(self, size: i32) -> Rc<dyn Yard> {
		PadYard::new(size, self)
	}
}

struct PadYard {
	id: i32,
	left_cols: i32,
	right_cols: i32,
	top_rows: i32,
	bottom_rows: i32,
	yard: Rc<dyn Yard>,
}

impl PadYard {
	fn new(size: i32, yard: Rc<dyn  Yard>) -> Rc<dyn Yard> {
		let cols = size * 2;
		let rows = size;
		Rc::new(PadYard {
			id: rand::random(),
			left_cols: cols,
			right_cols: cols,
			top_rows: rows,
			bottom_rows: rows,
			yard,
		})
	}
}

impl Yard for PadYard {
	fn id(&self) -> i32 { self.id }

	fn layout(&self, ctx: &mut dyn LayoutContext) -> usize {
		let (edge_index, edge_bounds) = ctx.edge_bounds();
		let alt_bounds = edge_bounds.pad(self.left_cols, self.right_cols, self.top_rows, self.bottom_rows);
		let alt_index = ctx.push_bounds(&alt_bounds);
		let mut alt_ctx = LayoutContextImpl::new(alt_index, ctx.bounds_hold().to_owned());
		let core_index = self.yard.layout(&mut alt_ctx);
		if core_index == alt_index {
			edge_index
		} else {
			let core_bounds = ctx.bounds(core_index);
			let final_bounds = edge_bounds.with_z(core_bounds.z);
			let final_index = ctx.push_bounds(&final_bounds);
			final_index
		}
	}

	fn render(&self, ctx: &dyn RenderContext) {
		self.yard.render(ctx)
	}
}