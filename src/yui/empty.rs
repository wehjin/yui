use std::rc::Rc;

use crate::yui::{RenderContext, Yard, YardOption};
use crate::yui::layout::LayoutContext;

pub fn empty_yard() -> Rc<dyn Yard> {
	Rc::new(EmptyYard { id: rand::random() })
}

struct EmptyYard {
	id: i32,
}

impl Yard for EmptyYard {
	fn id(&self) -> i32 {
		self.id
	}
	fn update(&self, _option: YardOption) {}

	fn layout(&self, ctx: &mut LayoutContext) -> usize {
		let (bounds_id, _bounds) = ctx.edge_bounds();
		bounds_id
	}

	fn render(&self, _ctx: &dyn RenderContext) {}
}