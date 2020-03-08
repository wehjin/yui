use std::rc::Rc;

use crate::yui::{Cling, Confine, LayoutContext, RenderContext, Yard};
use crate::yui::layout::LayoutContextImpl;

impl Confine for Rc<dyn Yard> {
	fn confine_height(self, height: i32, cling: Cling) -> Rc<dyn Yard> {
		ConfineYard::new(None, Some(height), cling, self)
	}

	fn confine(self, width: i32, height: i32, cling: Cling) -> Rc<dyn Yard> {
		ConfineYard::new(Some(width), Some(height), cling, self)
	}
}

struct ConfineYard {
	id: i32,
	width: Option<i32>,
	height: Option<i32>,
	cling: Cling,
	yard: Rc<dyn Yard>,
}

impl ConfineYard {
	fn new(width: Option<i32>, height: Option<i32>, cling: Cling, yard: Rc<dyn Yard>) -> Rc<dyn Yard> {
		Rc::new(ConfineYard {
			id: rand::random(),
			width,
			height,
			cling,
			yard,
		})
	}
}

impl Yard for ConfineYard {
	fn id(&self) -> i32 { self.id }

	fn layout(&self, ctx: &mut dyn LayoutContext) -> usize {
		let (edge_index, edge_bounds) = ctx.edge_bounds();
		let (width, height) = (
			self.width.unwrap_or_else(|| edge_bounds.width()),
			self.height.unwrap_or_else(|| edge_bounds.height())
		);
		let alt_bounds = edge_bounds.confine(width, height, self.cling);
		let mut alt_ctx = LayoutContextImpl::new(ctx.push_bounds(&alt_bounds), ctx.bounds_hold().to_owned());
		let core_index = self.yard.layout(&mut alt_ctx);
		if core_index == alt_ctx.current_index() {
			edge_index
		} else {
			let core_bounds = ctx.bounds(core_index);
			let final_bounds = edge_bounds.with_z(core_bounds.z);
			ctx.push_bounds(&final_bounds)
		}
	}

	fn render(&self, ctx: &dyn RenderContext) {
		self.yard.render(ctx)
	}
}
