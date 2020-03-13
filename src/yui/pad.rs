use std::sync::Arc;

use crate::yui::{ArcYard, Padding, RenderContext, Yard, YardOption};
use crate::yui::layout::LayoutContext;

impl Padding for ArcYard {
	fn pad(self, size: i32) -> ArcYard {
		PadYard::new(size, self)
	}
}

struct PadYard {
	id: i32,
	left_cols: i32,
	right_cols: i32,
	top_rows: i32,
	bottom_rows: i32,
	yard: ArcYard,
}

impl PadYard {
	fn new(size: i32, yard: ArcYard) -> ArcYard {
		let cols = size * 2;
		let rows = size;
		Arc::new(PadYard {
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
	fn update(&self, _option: YardOption) {}

	fn layout(&self, ctx: &mut LayoutContext) -> usize {
		let (edge_index, edge_bounds) = ctx.edge_bounds();
		let alt_bounds = edge_bounds.pad(self.left_cols, self.right_cols, self.top_rows, self.bottom_rows);
		let alt_index = ctx.push_bounds(&alt_bounds);
		let mut alt_ctx = ctx.with_index(alt_index);
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
