use std::sync::Arc;

use crate::{Bounds, DrawPad};
use crate::layout::LayoutContext;
use crate::palette::FillGrade;
use crate::yard::{ArcYard, Yard, YardOption};

pub fn grade(grade: FillGrade) -> ArcYard {
	//! Produce a yard that changes the color grade.
	Arc::new(GradeYard {
		id: rand::random(),
		grade,
	})
}

struct GradeYard {
	id: i32,
	grade: FillGrade,
}

impl Yard for GradeYard {
	fn id(&self) -> i32 { self.id }

	fn update(&self, _option: YardOption) {}

	fn layout(&self, ctx: &mut LayoutContext) -> usize {
		let (bounds_id, _bounds) = ctx.edge_bounds();
		ctx.set_yard_bounds(self.id, bounds_id);
		bounds_id
	}

	fn render(&self, bounds: &Bounds, _focus_id: i32, pad: &mut dyn DrawPad) -> Option<Vec<(ArcYard, Option<i32>)>> {
		pad.grade(bounds, self.grade);
		None
	}
}
