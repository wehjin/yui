use std::rc::Rc;

use crate::yui::{LayoutContext, RenderContext, Yard};

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

	fn layout(&self, ctx: &mut dyn LayoutContext) -> usize {
		let (bounds_id, _bounds) = ctx.edge_bounds();
		bounds_id
	}

	fn render(&self, _ctx: &dyn RenderContext) {}
}