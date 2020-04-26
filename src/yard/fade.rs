use std::sync::Arc;

use crate::{Fade, MultiLayout, RenderContext};
use crate::yard::{ArcYard, Yard, YardOption};
use crate::yui::layout::LayoutContext;

pub fn fade(indents: (i32, i32), rear_yard: ArcYard, fore_yard: ArcYard) -> ArcYard {
	Arc::new(FadeYard {
		id: rand::random(),
		indents,
		rear_yard,
		fore_yard,
	})
}

struct FadeYard {
	id: i32,
	indents: (i32, i32),
	rear_yard: ArcYard,
	fore_yard: ArcYard,
}

impl Fade for ArcYard {
	fn fade(self, indents: (i32, i32), fore_yard: ArcYard) -> ArcYard {
		fade(indents, self, fore_yard)
	}
}

impl Yard for FadeYard {
	fn id(&self) -> i32 {
		self.id
	}

	fn update(&self, _option: YardOption) {}

	fn layout(&self, ctx: &mut LayoutContext) -> usize {
		let (_bounds_id, bounds) = ctx.edge_bounds();
		let mut multi_layout = MultiLayout::new(ctx);
		multi_layout.layout(&self.rear_yard, &bounds);

		let (cols, rows) = self.indents;
		let indent_bounds = bounds.pad(cols, cols, rows, rows);
		let fore_z = multi_layout.near_z() - 1;
		let fore_bounds = indent_bounds.with_z(fore_z);
		multi_layout.layout(&self.fore_yard, &fore_bounds);

		let end_id = multi_layout.finish();
		ctx.set_yard_bounds(self.id(), end_id);
		ctx.set_focus_max(fore_z);
		end_id
	}

	fn render(&self, ctx: &dyn RenderContext) {
		let (row, col) = ctx.spot();
		let bounds = ctx.yard_bounds(self.id);
		if bounds.intersects(row, col) {
			let (cols, rows) = self.indents;
			let inside = bounds.pad(cols, cols, rows, rows);
			if inside.intersects(row, col) {
				self.fore_yard.render(ctx);
			} else {
				ctx.set_dark(bounds.z);
				self.rear_yard.render(ctx);
			}
		}
	}
}