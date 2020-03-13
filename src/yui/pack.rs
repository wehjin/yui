use std::cmp::min;
use std::sync::Arc;

use crate::yui::{ArcYard, PackTop, RenderContext, Yard, YardOption};
use crate::yui::layout::LayoutContext;

impl PackTop for ArcYard {
	fn pack_top(self, top_rows: i32, top_yard: ArcYard) -> ArcYard {
		Arc::new(PackYard {
			id: rand::random(),
			top_rows,
			top_yard,
			bottom_yard: self,
		})
	}
}

struct PackYard {
	id: i32,
	top_rows: i32,
	top_yard: ArcYard,
	bottom_yard: ArcYard,
}

impl Yard for PackYard {
	fn id(&self) -> i32 { self.id }
	fn update(&self, _option: YardOption) {}

	fn layout(&self, ctx: &mut LayoutContext) -> usize {
		let (edge_index, edge_bounds) = ctx.edge_bounds();
		let (top, bottom) = edge_bounds.split_from_top(self.top_rows);
		let (top_pre_index, bottom_pre_index) = (ctx.push_bounds(&top), ctx.push_bounds(&bottom));
		let (mut top_ctx, mut bottom_ctx) = (ctx.with_index(top_pre_index), ctx.with_index(bottom_pre_index));
		let (top_layout_index, bottom_layout_index) = (self.top_yard.layout(&mut top_ctx), self.bottom_yard.layout(&mut bottom_ctx));
		let (top_layout_bounds, bottom_layout_bounds) = (ctx.bounds(top_layout_index), ctx.bounds(bottom_layout_index));
		let min_z = min(top_layout_bounds.z, bottom_layout_bounds.z);
		let final_index = if edge_bounds.z == min_z { edge_index } else { ctx.push_bounds(&edge_bounds.with_z(min_z)) };
		final_index
	}

	fn render(&self, ctx: &dyn RenderContext) {
		self.top_yard.render(ctx);
		self.bottom_yard.render(ctx);
	}
}
