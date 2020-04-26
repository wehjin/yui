use std::sync::Arc;

use crate::yard::{ArcYard, Yard, YardOption};
use crate::yui::{Cling, Place, RenderContext};
use crate::yui::bounds::Bounds;
use crate::yui::layout::LayoutContext;

impl Place for ArcYard {
	fn place_center(self, width: i32) -> ArcYard {
		Arc::new(PlaceYard {
			id: rand::random(),
			core_yard: self,
			position: Arc::new(move |bounds| {
				bounds.confine(width, bounds.height(), Cling::CenterMiddle)
			}),
		})
	}
}

struct PlaceYard {
	id: i32,
	core_yard: ArcYard,
	position: Arc<dyn Fn(&Bounds) -> Bounds + Send + Sync>,
}

impl Yard for PlaceYard {
	fn id(&self) -> i32 { self.id }
	fn update(&self, option: YardOption) { self.core_yard.update(option) }

	fn layout(&self, ctx: &mut LayoutContext) -> usize {
		let (edge_index, edge_bounds) = ctx.edge_bounds();
		let core = (*self.position)(&edge_bounds);
		let core_index = ctx.push_bounds(&core);
		let mut core_ctx = ctx.with_index(core_index);
		let core_layout_index = self.core_yard.layout(&mut core_ctx);
		let core_layout_bounds = ctx.bounds(core_layout_index);
		let min_z = core_layout_bounds.z;
		let final_index = if edge_bounds.z == min_z { edge_index } else { ctx.push_bounds(&edge_bounds.with_z(min_z)) };
		final_index
	}

	fn render(&self, ctx: &dyn RenderContext) {
		self.core_yard.render(ctx)
	}
}
