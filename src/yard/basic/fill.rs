use std::sync::Arc;

use crate::{Bounds, DrawPad};
use crate::layout::LayoutContext;
use crate::palette::{FillColor, FillGrade};
use crate::yard::{ArcYard, Yard};

pub fn fill(color: FillColor, grade: FillGrade) -> ArcYard {
	//! Produce a yard that renders as a rectangle filled the specified color.
	Arc::new(FillYard {
		id: rand::random(),
		color: (color, grade),
	})
}

pub fn fill_plain_background() -> ArcYard {
	fill(FillColor::Background, FillGrade::Plain)
}

struct FillYard {
	id: i32,
	color: (FillColor, FillGrade),
}

impl Yard for FillYard {
	fn id(&self) -> i32 { self.id }
	fn type_desc(&self) -> &'static str { "Fill" }

	fn layout(&self, ctx: &mut LayoutContext) -> usize {
		let (bounds_id, _bounds) = ctx.edge_bounds();
		ctx.set_yard_bounds(self.id, bounds_id);
		bounds_id
	}

	fn render(&self, bounds: &Bounds, _focus_id: i32, pad: &mut dyn DrawPad) -> Option<Vec<(ArcYard, Option<i32>)>> {
		let (color, grade) = self.color;
		pad.grade(bounds, grade);
		pad.fill(bounds, color);
		None
	}
}