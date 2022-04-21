use std::sync::Arc;

use crate::{Bounds, DrawPad};
use crate::layout::LayoutContext;
use crate::palette::StrokeColor;
use crate::yard::{ArcYard, Yard};

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

	fn layout(&self, ctx: &mut LayoutContext) -> usize {
		let (bounds_id, _bounds) = ctx.edge_bounds();
		ctx.set_yard_bounds(self.id(), bounds_id);
		bounds_id
	}

	fn render(&self, bounds: &Bounds, _focus_id: i32, pad: &mut dyn DrawPad) -> Option<Vec<(ArcYard, Option<i32>)>> {
		let glyph = (*self.glyph)();
		if !glyph.is_control() {
			let width = bounds.width();
			if width > 0 {
				let multi_glyph: String = (0..width).into_iter().map(|_| glyph).collect();
				pad.glyph(bounds, &multi_glyph, self.color);
			}
		}
		None
	}
}

#[cfg(test)]
mod tests {
	use crate::{layout, render, StrokeColor, yard};
	use crate::StrokeColor::BodyOnBackground;
	use crate::yui::layout::ActiveFocus;

	#[test]
	fn layout_render() {
		let yard = yard::glyph(StrokeColor::BodyOnBackground, || '=');
		let (max_x, max_y) = (2, 1);
		let layout = layout::run(max_y, max_x, &yard, &ActiveFocus::default());
		let draw_pad = render::run(&yard, layout.max_x, layout.max_y, layout.bounds_hold.clone(), 0);
		let fronts = draw_pad.to_fronts();
		let fills = fronts.iter().flatten()
			.map(|front| front.stroke.clone().unwrap_or((" ".to_string(), StrokeColor::BodyOnBackground)))
			.collect::<Vec<_>>();
		assert_eq!(fills, vec![("=".to_string(), BodyOnBackground), ("=".to_string(), BodyOnBackground)])
	}
}