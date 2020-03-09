use std::cell::Cell;
use std::rc::Rc;

use crate::yui::{RenderContext, Yard, YardOption};
use crate::yui::layout::LayoutContext;
use crate::yui::palette::FillColor;

pub fn fill_yard(color: FillColor) -> Rc<dyn Yard> {
	Rc::new(FillYard {
		id: rand::random(),
		color: Cell::new(color),
	})
}

struct FillYard {
	id: i32,
	color: Cell<FillColor>,
}

impl Yard for FillYard {
	fn id(&self) -> i32 {
		self.id
	}

	fn update(&self, option: YardOption) {
		let YardOption::FillColor(color) = option;
		self.color.set(color)
	}

	fn layout(&self, ctx: &mut LayoutContext) -> usize {
		let (bounds_id, _bounds) = ctx.edge_bounds();
		ctx.set_yard_bounds(self.id(), bounds_id);
		bounds_id
	}

	fn render(&self, ctx: &dyn RenderContext) {
		let (row, col) = ctx.spot();
		let bounds = ctx.yard_bounds(self.id);
		if bounds.intersects(row, col) {
			ctx.set_fill(self.color.get(), bounds.z)
		}
	}
}