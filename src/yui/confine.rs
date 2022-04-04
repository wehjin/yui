use std::sync::Arc;

use crate::{Bounds, DrawPad};
use crate::layout::LayoutContext;
use crate::yard::{ArcYard, Yard, YardOption};
use crate::yui::{Cling, Confine};

impl Confine for ArcYard {
	fn confine_height(self, height: i32, cling: Cling) -> ArcYard {
		ConfineYard::new(None, Some(height), cling, self)
	}

	fn confine_width(self, width: i32, cling: Cling) -> ArcYard {
		ConfineYard::new(Some(width), None, cling, self)
	}

	fn confine(self, width: i32, height: i32, cling: Cling) -> ArcYard {
		ConfineYard::new(Some(width), Some(height), cling, self)
	}
}

struct ConfineYard {
	id: i32,
	width: Option<i32>,
	height: Option<i32>,
	cling: Cling,
	yard: ArcYard,
}

impl ConfineYard {
	fn new(width: Option<i32>, height: Option<i32>, cling: Cling, yard: ArcYard) -> ArcYard {
		Arc::new(ConfineYard {
			id: rand::random(),
			width,
			height,
			cling,
			yard,
		})
	}
}

impl Yard for ConfineYard {
	fn id(&self) -> i32 { self.id }
	fn type_desc(&self) -> &'static str { "Confine" }
	fn update(&self, _option: YardOption) {}

	fn layout(&self, ctx: &mut LayoutContext) -> usize {
		let (edge_index, edge_bounds) = { ctx.edge_bounds() };
		let (width, height) = (
			self.width.unwrap_or_else(|| edge_bounds.width()),
			self.height.unwrap_or_else(|| edge_bounds.height())
		);
		let alt_bounds = edge_bounds.confine(width, height, self.cling);
		let alt_index = { ctx.push_bounds(&alt_bounds) };
		let mut alt_ctx = ctx.with_index(alt_index);
		let core_index = self.yard.layout(&mut alt_ctx);
		if core_index == alt_ctx.current_index() {
			edge_index
		} else {
			let core_bounds = ctx.bounds(core_index);
			let final_bounds = edge_bounds.with_z(core_bounds.z);
			ctx.push_bounds(&final_bounds)
		}
	}

	fn render(&self, _bounds: &Bounds, _focus_id: i32, _pad: &mut dyn DrawPad) -> Option<Vec<(ArcYard, Option<i32>)>> {
		Some(vec![(self.yard.clone(), None)])
	}
}
