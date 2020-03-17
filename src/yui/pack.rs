use std::cmp::min;
use std::sync::Arc;

use crate::yui::{ArcYard, Pack, RenderContext, Yard, YardOption};
use crate::yui::bounds::Bounds;
use crate::yui::layout::LayoutContext;

impl Pack for ArcYard {
	fn pack_top(self, top_rows: i32, top_yard: ArcYard) -> ArcYard {
		Arc::new(PackYard {
			id: rand::random(),
			first_yard: top_yard,
			second_yard: self,
			divide: Arc::new(move |bounds| bounds.split_from_top(top_rows)),
		})
	}

	fn pack_bottom(self, rows: i32, bottom_yard: ArcYard) -> ArcYard {
		Arc::new(PackYard {
			id: rand::random(),
			first_yard: self,
			second_yard: bottom_yard,
			divide: Arc::new(move |bounds| bounds.split_from_bottom(rows)),
		})
	}

	fn pack_right(self, cols: i32, right_yard: ArcYard) -> ArcYard {
		Arc::new(PackYard {
			id: rand::random(),
			first_yard: self,
			second_yard: right_yard,
			divide: Arc::new(move |bounds| bounds.split_from_right(cols)),
		})
	}
}

struct PackYard {
	id: i32,
	first_yard: ArcYard,
	second_yard: ArcYard,
	divide: Arc<dyn Fn(Bounds) -> (Bounds, Bounds) + Send + Sync>,
}

impl Yard for PackYard {
	fn id(&self) -> i32 { self.id }
	fn update(&self, _option: YardOption) {}

	fn layout(&self, ctx: &mut LayoutContext) -> usize {
		let (edge_index, edge_bounds) = ctx.edge_bounds();
		let (first, second) = (*self.divide)(edge_bounds);
		let (first_index, second_index) = (ctx.push_bounds(&first), ctx.push_bounds(&second));
		let (mut first_ctx, mut second_ctx) = (ctx.with_index(first_index), ctx.with_index(second_index));
		let (first_layout_index, second_layout_index) = (self.first_yard.layout(&mut first_ctx), self.second_yard.layout(&mut second_ctx));
		let (first_layout_bounds, second_layout_bounds) = (ctx.bounds(first_layout_index), ctx.bounds(second_layout_index));
		let min_z = min(first_layout_bounds.z, second_layout_bounds.z);
		let final_index = if edge_bounds.z == min_z { edge_index } else { ctx.push_bounds(&edge_bounds.with_z(min_z)) };
		final_index
	}

	fn render(&self, ctx: &dyn RenderContext) {
		self.first_yard.render(ctx);
		self.second_yard.render(ctx);
	}
}
