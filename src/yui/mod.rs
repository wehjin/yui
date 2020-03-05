use std::cell::RefCell;
use std::rc::Rc;

use crate::yui::bounds::{Bounds, BoundsHold};

pub mod fill;
pub mod pad;
pub mod bounds;
pub mod layout;

pub trait Yard {
	fn yard_id(&self) -> i32;
	fn layout(&self, ctx: &mut dyn LayoutContext) -> usize;
	fn render(&self, ctx: &dyn RenderContext);
}

pub trait LayoutContext {
	fn bounds_hold(&self) -> Rc<RefCell<BoundsHold>>;
	fn edge_bounds(&self) -> (usize, Bounds);
	fn push_core_bounds(&mut self, bounds: &Bounds) -> usize;
	fn set_yard_bounds(&mut self, yard_id: i32, bounds_index: usize);
}

pub trait RenderContext {
	fn spot(&self) -> (i32, i32);
	fn yard_bounds(&self, yard_id: i32) -> Bounds;
	fn set_fill(&self, row: i32, col: i32);
}

pub trait Padding {
	fn pad_sides(self, size: i32) -> Rc<dyn Yard>;
}
