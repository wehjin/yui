use std::sync::Arc;

use unicode_width::UnicodeWidthStr;

use crate::{Bounds, Cling, DrawPad};
use crate::layout::LayoutContext;
use crate::palette::StrokeColor;
use crate::yard::{ArcYard, Yard, YardOption};

pub fn label<S: AsRef<str>>(string: S, color: StrokeColor, cling: Cling) -> ArcYard {
	//! Generate a yard that displays a string of characters.
	let id = rand::random();
	let string = string.as_ref().chars().filter(|it| !it.is_control()).collect::<String>();
	let string_width = UnicodeWidthStr::width(string.as_str());
	Arc::new(LabelYard { id, color, string, string_width, cling })
}

struct LabelYard {
	id: i32,
	color: StrokeColor,
	string: String,
	string_width: usize,
	cling: Cling,
}

impl Yard for LabelYard {
	fn id(&self) -> i32 { self.id }
	fn type_desc(&self) -> &'static str { "Label" }

	fn update(&self, _option: YardOption) {}

	fn layout(&self, ctx: &mut LayoutContext) -> usize {
		let (bounds_id, _bounds) = ctx.edge_bounds();
		ctx.set_yard_bounds(self.id(), bounds_id);
		bounds_id
	}

	fn render(&self, bounds: &Bounds, _focus_id: i32, pad: &mut dyn DrawPad) -> Option<Vec<(ArcYard, Option<i32>)>> {
		let (extra_width, extra_height) = (bounds.width() - self.string_width as i32, bounds.height() - 1);
		let (cling_x, cling_y) = self.cling.into();
		let (extra_left, extra_top) = ((extra_width as f32 * cling_x) as i32, (extra_height as f32 * cling_y) as i32);
		let extra_bottom = bounds.height() - extra_top - 1;
		let text_bounds = bounds.pad(extra_left, 0, extra_top, extra_bottom);
		info!("RENDERING LABEL to bounds: {:?}, text-bounds: {:?}", bounds,  text_bounds);
		pad.glyph(&text_bounds, &self.string, self.color);
		None
	}
}

#[cfg(test)]
mod tests {
	use crate::{Cling, layout, render, SenderLink, StrokeColor, yard};
	use crate::StrokeColor::BodyOnBackground;
	use crate::yui::layout::ActiveFocus;

	#[test]
	fn layout_render() {
		let yard = yard::label("Hi", StrokeColor::BodyOnBackground, Cling::Left);
		let (max_x, max_y) = (2, 1);
		let layout = layout::run(max_y, max_x, &yard, SenderLink::ignore(), &ActiveFocus::default());
		let spot_table = render::run(yard.clone(), layout.max_x, layout.max_y, layout.bounds.clone(), layout.active_focus.focus_id());
		let fronts = spot_table.to_fronts();
		let computed = fronts.iter().flatten()
			.map(|front| front.stroke.clone().unwrap_or((" ".to_string(), StrokeColor::BodyOnBackground)))
			.collect::<Vec<_>>();
		assert_eq!(computed, vec![("H".to_string(), BodyOnBackground), ("i".to_string(), BodyOnBackground)])
	}
}