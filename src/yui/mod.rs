use std::cell::RefCell;
use std::rc::Rc;

use crate::yui::bounds::{Bounds, BoundsHold};
use crate::yui::palette::{FillColor, StrokeColor};

pub mod bounds;
pub mod layout;
pub mod glyph;
pub mod fill;
pub mod pad;
pub mod before;
pub mod palette;
pub mod pack;
pub mod label;
pub mod button;
pub mod confine;
pub mod empty;

pub trait Yard {
	fn id(&self) -> i32;
	fn layout(&self, ctx: &mut dyn LayoutContext) -> usize;
	fn render(&self, ctx: &dyn RenderContext);
}

pub trait LayoutContext {
	fn current_index(&self) -> usize;
	fn bounds_hold(&self) -> Rc<RefCell<BoundsHold>>;
	fn edge_bounds(&self) -> (usize, Bounds);
	fn bounds(&self, index: usize) -> Bounds;
	fn push_bounds(&mut self, bounds: &Bounds) -> usize;
	fn set_yard_bounds(&mut self, yard_id: i32, bounds_index: usize);
}

pub trait RenderContext {
	fn spot(&self) -> (i32, i32);
	fn yard_bounds(&self, yard_id: i32) -> Bounds;
	fn set_fill(&self, color: FillColor, z: i32);
	fn set_glyph(&self, glyph: char, color: StrokeColor, z: i32);
}

pub trait Padding {
	fn pad(self, size: i32) -> Rc<dyn Yard>;
}

pub trait Confine {
	fn confine_height(self, height: i32, cling: Cling) -> Rc<dyn Yard>;
	fn confine(self, width: i32, height: i32, cling: Cling) -> Rc<dyn Yard>;
}

pub trait Before {
	fn before(self, yard: Rc<dyn Yard>) -> Rc<dyn Yard>;
}

pub trait PackTop {
	fn pack_top(self, rows: i32, top_yard: Rc<dyn Yard>) -> Rc<dyn Yard>;
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Cling {
	Custom { x: f32, y: f32 },
	CenterMiddle,
}

impl Cling {
	fn x(&self) -> f32 {
		match self {
			Cling::Custom { x, .. } => { x.to_owned() }
			Cling::CenterMiddle => { 0.5 }
		}
	}
	fn y(&self) -> f32 {
		match self {
			Cling::Custom { y, .. } => { y.to_owned() }
			Cling::CenterMiddle => { 0.5 }
		}
	}
}

impl From<Cling> for (f32, f32) {
	fn from(cling: Cling) -> Self {
		(cling.x(), cling.y())
	}
}
