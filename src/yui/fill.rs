use std::rc::Rc;

use crate::yui::{LayoutContext, RenderContext, Yard};
use crate::yui::palette::FillColor;

pub fn fill_yard(color: FillColor) -> Rc<dyn Yard> {
	Rc::new(FillYard { id: rand::random(), color })
}

struct FillYard {
	id: i32,
	color: FillColor,
}

impl Yard for FillYard {
	fn id(&self) -> i32 {
		self.id
	}

	fn layout(&self, ctx: &mut dyn LayoutContext) -> usize {
		let (bounds_id, _bounds) = ctx.edge_bounds();
		ctx.set_yard_bounds(self.id(), bounds_id);
		bounds_id
	}

	fn render(&self, ctx: &dyn RenderContext) {
		let (row, col) = ctx.spot();
		let bounds = ctx.yard_bounds(self.id);
		if bounds.intersects(row, col) {
			ctx.set_fill(self.color, bounds.z)
		}
	}
}