use std::rc::Rc;

use crate::yui::{Cling, RenderContext, Yard, YardOption};
use crate::yui::palette::StrokeColor;
use crate::yui::layout::LayoutContext;

pub fn label_yard(string: &str, color: StrokeColor, cling: Cling) -> Rc<dyn Yard> {
	Rc::new(LabelYard { id: rand::random(), color, string: string.to_owned(), cling })
}

struct LabelYard {
	id: i32,
	color: StrokeColor,
	string: String,
	cling: Cling,
}

impl Yard for LabelYard {
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
			let chars: Vec<char> = self.string.chars().filter(|it| it.is_ascii() && !it.is_control()).collect();
			let (extra_width, extra_height) = (bounds.width() - chars.len() as i32, bounds.height() - 1);
			let (x, y) = self.cling.into();
			let (extra_left, extra_top) = ((extra_width as f32 * x) as i32, (extra_height as f32 * y) as i32);
			let (left_indent, top_indent) = (col - bounds.left, row - bounds.top);
			let line_indent = top_indent - extra_top;
			let string_indent = left_indent - extra_left;
			let glyph = if line_indent != 0 || string_indent < 0 || string_indent as usize >= chars.len() {
				' '
			} else {
				chars[string_indent as usize]
			};
			ctx.set_glyph(glyph, self.color, bounds.z);
		}
	}
}