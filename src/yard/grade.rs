use std::sync::Arc;

use crate::palette::FillGrade;
use crate::RenderContext;
use crate::yard::{ArcYard, Yard, YardOption};
use crate::yui::layout::LayoutContext;

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
	fn render(&self, ctx: &dyn RenderContext) {
		let (row, col) = ctx.spot();
		let bounds = ctx.yard_bounds(self.id);
		if bounds.intersects(row, col) {
			ctx.set_fill_grade(self.grade, bounds.z);
		}
	}

	fn layout(&self, ctx: &mut LayoutContext) -> usize {
		let (bounds_id, _bounds) = ctx.edge_bounds();
		ctx.set_yard_bounds(self.id, bounds_id);
		bounds_id
	}

	fn update(&self, _option: YardOption) {}

	fn id(&self) -> i32 { self.id }
}
