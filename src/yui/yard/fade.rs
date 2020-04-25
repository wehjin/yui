use std::sync::Arc;

use crate::Fade;
use crate::yui::{ArcYard, RenderContext, Yard, YardOption};
use crate::yui::layout::LayoutContext;
use crate::yui::palette::FillColor;

pub fn fade(indents: (i32, i32), rear_yard: ArcYard) -> ArcYard {
	Arc::new(FadeYard { id: rand::random(), indents, rear_yard })
}

struct FadeYard {
	id: i32,
	indents: (i32, i32),
	rear_yard: ArcYard,
}

impl Fade for ArcYard {
	fn fade(self, indents: (i32, i32)) -> ArcYard {
		fade(indents, self)
	}
}

impl Yard for FadeYard {
	fn id(&self) -> i32 {
		self.id
	}

	fn update(&self, _option: YardOption) {}

	fn layout(&self, ctx: &mut LayoutContext) -> usize {
		let (_bounds_id, bounds) = ctx.edge_bounds();
		let rear_id = self.rear_yard.layout(ctx);
		let rear_z = ctx.bounds(rear_id).z;
		let fore_bounds = bounds.with_z(rear_z - 1);
		let fore_id = ctx.push_bounds(&fore_bounds);
		ctx.set_yard_bounds(self.id(), fore_id);
		ctx.set_focus_max(fore_bounds.z);
		fore_id
	}

	fn render(&self, ctx: &dyn RenderContext) {
		let (row, col) = ctx.spot();
		let bounds = ctx.yard_bounds(self.id);
		if bounds.intersects(row, col) {
			let (cols, rows) = self.indents;
			let no_fade = bounds.pad(cols, cols, rows, rows);
			if no_fade.intersects(row, col) {
				ctx.set_fill(FillColor::Background, bounds.z);
			} else {
				ctx.set_dark(bounds.z);
				self.rear_yard.render(ctx);
			}
		}
	}
}