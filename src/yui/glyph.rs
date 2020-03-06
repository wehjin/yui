use std::rc::Rc;

use crate::yui::{LayoutContext, RenderContext, Yard};

pub fn glyph_yard(glyph: char) -> Rc<dyn Yard> {
	assert!(!glyph.is_control());
	let id = rand::random();
	Rc::new(GlyphYard { id, glyph })
}

struct GlyphYard {
	id: i32,
	glyph: char,
}

impl Yard for GlyphYard {
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
			ctx.set_glyph(self.glyph, bounds.near);
		}
	}
}