use std::sync::Arc;

use crate::{Bounds, DrawPad};
use crate::layout::LayoutContext;
use crate::yard::{ArcYard, Yard, YardOption};

pub fn empty() -> ArcYard {
	Arc::new(EmptyYard { id: rand::random() })
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

	fn render(&self, _bounds: &Bounds, _focus_id: i32, _pad: &mut dyn DrawPad) -> Option<Vec<(ArcYard, Option<i32>)>> {
		Option::None
	}
}