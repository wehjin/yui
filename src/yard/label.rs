use std::sync::Arc;

use unicode_width::UnicodeWidthStr;

use crate::{Cling, RenderContext};
use crate::palette::StrokeColor;
use crate::yard::{ArcYard, Yard, YardOption};
use crate::yui::layout::LayoutContext;

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
	fn render(&self, ctx: &dyn RenderContext) {
		let (row, col) = ctx.spot();
		let bounds = ctx.yard_bounds(self.id);
		if bounds.intersects(row, col) {
			let (extra_width, extra_height) = (bounds.width() - self.string_width as i32, bounds.height() - 1);
			let (cling_x, cling_y) = self.cling.into();
			let (extra_left, extra_top) = ((extra_width as f32 * cling_x) as i32, (extra_height as f32 * cling_y) as i32);
			let (left_indent, top_indent) = (col - bounds.left, row - bounds.top);
			let line_indent = top_indent - extra_top;
			let string_indent = left_indent - extra_left;
			let glyph = if line_indent != 0 || string_indent < 0 || string_indent >= self.string_width as i32 {
				" ".to_string()
			} else if string_indent == 0 {
				self.string.clone()
			} else {
				"".to_string()
			};
			ctx.set_glyph(glyph, self.color, bounds.z);
		}
	}
	fn layout(&self, ctx: &mut LayoutContext) -> usize {
		let (bounds_id, _bounds) = ctx.edge_bounds();
		ctx.set_yard_bounds(self.id(), bounds_id);
		bounds_id
	}

	fn update(&self, _option: YardOption) {}

	fn id(&self) -> i32 {
		self.id
	}
}