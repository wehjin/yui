use std::sync::Arc;

use crate::yard::{ArcYard, Yard, YardOption};
use crate::yui::RenderContext;
use crate::yui::layout::LayoutContext;

pub fn empty_yard() -> ArcYard {
	Arc::new(EmptyYard { id: rand::random() })
}

struct EmptyYard {
	id: i32,
}

impl Yard for EmptyYard {
	fn id(&self) -> i32 {
		self.id
	}
	fn update(&self, _option: YardOption) {}

	fn layout(&self, ctx: &mut LayoutContext) -> usize {
		let (bounds_id, _bounds) = ctx.edge_bounds();
		bounds_id
	}

	fn render(&self, _ctx: &dyn RenderContext) {}
}