use std::rc::Rc;

use crate::yui::{LayoutContext, RenderContext, Yard};
use crate::yui::palette::StrokeColor;

pub fn label_yard(string: &str, color: StrokeColor) -> Rc<dyn Yard> {
	Rc::new(LabelYard { id: rand::random(), color, string: string.to_owned() })
}

struct LabelYard {
	id: i32,
	color: StrokeColor,
	string: String,
}

impl Yard for LabelYard {
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
			let chars: Vec<char> = self.string.chars().filter(|it| it.is_ascii() && !it.is_control()).collect();
			let extra_width = bounds.width() - chars.len() as i32;
			let anchor = 0.0;
			let extra_left = (extra_width as f32 * anchor) as i32;
			let left_indent = col - bounds.left;
			let string_indent = left_indent - extra_left;
			let glyph = if string_indent < 0 || string_indent as usize >= chars.len() {
				' '
			} else {
				chars[string_indent as usize]
			};
			ctx.set_glyph(glyph, self.color, bounds.z);
		}
	}
}