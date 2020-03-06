use std::cell::RefCell;
use std::rc::Rc;

use crate::yui::bounds::{Bounds, BoundsHold};

pub mod bounds;
pub mod layout;
pub mod glyph;
pub mod fill;
pub mod pad;

pub trait Yard {
	fn id(&self) -> i32;
	fn layout(&self, ctx: &mut dyn LayoutContext) -> usize;
	fn render(&self, ctx: &dyn RenderContext);
}

pub trait LayoutContext {
	fn bounds_hold(&self) -> Rc<RefCell<BoundsHold>>;
	fn edge_bounds(&self) -> (usize, Bounds);
	fn push_core_bounds(&mut self, bounds: &Bounds) -> usize;
	fn set_yard_bounds(&mut self, id: i32, bounds_index: usize);
}

pub trait RenderContext {
	fn spot(&self) -> (i32, i32);
	fn yard_bounds(&self, id: i32) -> Bounds;
	fn set_fill(&self, z: i32);
	fn set_glyph(&self, glyph: char, z: i32);
}

pub trait Padding {
	fn pad_sides(self, size: i32) -> Rc<dyn Yard>;
}
