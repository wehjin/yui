use std::rc::Rc;

use crate::yui::{Before, LayoutContext, RenderContext, Yard};
use crate::yui::layout::LayoutContextImpl;

impl Before for Rc<dyn Yard> {
	fn before(self, far_yard: Rc<dyn Yard>) -> Rc<dyn Yard> {
		BeforeYard::new(self, far_yard)
	}
}

struct BeforeYard {
	id: i32,
	near_yard: Rc<dyn Yard>,
	far_yard: Rc<dyn Yard>,
}

impl BeforeYard {
	fn new(near_yard: Rc<dyn Yard>, far_yard: Rc<dyn  Yard>) -> Rc<dyn Yard> {
		Rc::new(BeforeYard {
			id: rand::random(),
			near_yard,
			far_yard,
		})
	}
}

impl Yard for BeforeYard {
	fn id(&self) -> i32 { self.id }

	fn layout(&self, ctx: &mut dyn LayoutContext) -> usize {
		let (edge_index, edge_bounds) = ctx.edge_bounds();
		let far_index = self.far_yard.layout(ctx);
		let far_z = if far_index == edge_index {
			edge_bounds.z
		} else {
			ctx.bounds(far_index).z
		};
		let near_z = far_z - 1;
		let near_bounds = edge_bounds.with_z(near_z);
		let near_index = ctx.push_bounds(&near_bounds);
		let mut near_ctx = LayoutContextImpl::new(near_index, ctx.bounds_hold().clone());
		let final_index = self.near_yard.layout(&mut near_ctx);
		final_index
	}

	fn render(&self, ctx: &dyn RenderContext) {
		self.far_yard.render(ctx);
		self.near_yard.render(ctx);
	}
}
