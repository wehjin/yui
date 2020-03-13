use std::sync::Arc;

use crate::yui::{ArcYard, RenderContext, Yard, YardOption};
use crate::yui::layout::LayoutContext;
use crate::yui::palette::StrokeColor;

pub fn glyph_yard(glyph: char, color: StrokeColor) -> ArcYard {
	assert!(!glyph.is_control());
	let id = rand::random();
	Arc::new(GlyphYard { id, glyph, color })
}

struct GlyphYard {
	id: i32,
	glyph: char,
	color: StrokeColor,
}

impl Yard for GlyphYard {
	fn id(&self) -> i32 {
		self.id
	}
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
			ctx.set_glyph(self.glyph, self.color, bounds.z);
		}
	}
}