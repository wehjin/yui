use std::sync::Arc;

use crate::RenderContext;
use crate::yard::{ArcYard, Yard, YardOption};
use crate::yui::layout::LayoutContext;
use crate::yui::palette::StrokeColor;

pub fn glyph(color: StrokeColor, glyph: impl Fn() -> char + Send + Sync + 'static) -> ArcYard {
	Arc::new(GlyphYard {
		id: rand::random(),
		color,
		glyph: Arc::new(glyph),
	})
}

struct GlyphYard {
	id: i32,
	color: StrokeColor,
	glyph: Arc<dyn Fn() -> char + Send + Sync>,
}

impl Yard for GlyphYard {
	fn id(&self) -> i32 { self.id }
	fn update(&self, _option: YardOption) {}

	fn layout(&self, ctx: &mut LayoutContext) -> usize {
		let (bounds_id, _bounds) = ctx.edge_bounds();
		ctx.set_yard_bounds(self.id(), bounds_id);
		bounds_id
	}

	fn render(&self, ctx: &dyn RenderContext) {
		let (row, col) = ctx.spot();
		let bounds = ctx.yard_bounds(self.id);
		if bounds.intersects(row, col) {
			let glyph = (*self.glyph)();
			if !glyph.is_control() {
				ctx.set_glyph(glyph, self.color, bounds.z);
			}
		}
	}
}