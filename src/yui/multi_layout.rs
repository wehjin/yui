use std::rc::Rc;

use crate::Focus;
use crate::yard::ArcYard;
use crate::yui::bounds::Bounds;
use crate::yui::layout::LayoutContext;

pub struct MultiLayout<'a> {
	ctx: &'a mut LayoutContext,
	start_index: usize,
	start_bounds: Bounds,
	near_z: i32,
	finished: bool,
	trap_foci: bool,
	trapped_focus: Option<Rc<Focus>>,
}

impl<'a> MultiLayout<'a> {
	pub fn trap_foci(&mut self, value: bool) { self.trap_foci = value }
	pub fn trapped_focus(&self) -> Option<Rc<Focus>> {
		if let Some(ref focus) = self.trapped_focus {
			Some(focus.clone())
		} else {
			None
		}
	}
	pub fn near_z(&self) -> i32 { self.near_z }
	pub fn layout(&mut self, yard: &ArcYard, bounds: &Bounds) {
		assert!(!self.finished);
		self.trapped_focus = None;
		let different_bounds = bounds != &self.start_bounds;
		let out_index = if different_bounds || self.trap_foci {
			let ctx = if different_bounds {
				let in_index = self.ctx.push_bounds(bounds);
				self.ctx.with_index(in_index)
			} else {
				self.ctx.clone()
			};
			let mut ctx = if self.trap_foci {
				ctx.trap_foci()
			} else {
				ctx
			};
			let index = yard.layout(&mut ctx);
			if self.trap_foci {
				self.trapped_focus = ctx.trapped_focus()
			}
			index
		} else {
			yard.layout(self.ctx)
		};
		let out_z = self.ctx.bounds(out_index).z;
		if out_z < self.near_z {
			self.near_z = out_z;
		}
	}

	pub fn finish(&mut self) -> usize {
		assert!(!self.finished);
		self.finished = true;
		if self.near_z < self.start_bounds.z {
			let bounds = self.start_bounds.with_z(self.near_z);
			self.ctx.push_bounds(&bounds)
		} else {
			self.start_index
		}
	}

	pub fn new(ctx: &'a mut LayoutContext) -> Self {
		let (index, bounds) = ctx.edge_bounds();
		MultiLayout { ctx, start_index: index, start_bounds: bounds, near_z: bounds.z, finished: false, trap_foci: false, trapped_focus: None }
	}
}
