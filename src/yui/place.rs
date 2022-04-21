use std::sync::Arc;

use crate::DrawPad;
use crate::layout::LayoutContext;
use crate::yard::{ArcYard, Yard};
use crate::yui::{Cling, Place};
use crate::yui::bounds::Bounds;

impl Place for ArcYard {
	fn place_center(self, width: i32) -> ArcYard {
		Arc::new(PlaceYard {
			id: rand::random(),
			core_yard: self,
			position: Arc::new(move |bounds| {
				bounds.confine(width, bounds.height(), Cling::Center)
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
	fn type_desc(&self) -> &'static str { "Place" }

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

	fn render(&self, _bounds: &Bounds, _focus_id: i32, _pad: &mut dyn DrawPad) -> Option<Vec<(ArcYard, Option<i32>)>> {
		Some(vec![
			(self.core_yard.clone(), None)
		])
	}
}
