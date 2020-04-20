use std::sync::Arc;

use crate::yui::{ArcYard, Before, MultiLayout, RenderContext, Yard, YardOption};
use crate::yui::layout::LayoutContext;

impl Before for ArcYard {
	fn before(self, far_yard: ArcYard) -> ArcYard {
		BeforeYard::new(self, far_yard)
	}
}

struct BeforeYard {
	id: i32,
	near_yard: ArcYard,
	far_yard: ArcYard,
}

impl BeforeYard {
	fn new(near_yard: ArcYard, far_yard: ArcYard) -> ArcYard {
		Arc::new(BeforeYard {
			id: rand::random(),
			near_yard,
			far_yard,
		})
	}
}

impl Yard for BeforeYard {
	fn id(&self) -> i32 { self.id }
	fn update(&self, _option: YardOption) {}

	fn layout(&self, ctx: &mut LayoutContext) -> usize {
		let (_edge_index, edge_bounds) = ctx.edge_bounds();
		let mut multi_layout = MultiLayout::new(ctx);
		multi_layout.layout(&self.far_yard, &edge_bounds);
		multi_layout.layout(&self.near_yard, &edge_bounds.with_z(multi_layout.near_z() - 1));
		multi_layout.finish()
	}

	fn render(&self, ctx: &dyn RenderContext) {
		self.far_yard.render(ctx);
		self.near_yard.render(ctx);
	}
}
