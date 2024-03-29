use std::sync::Arc;

use crate::{Bounds, DrawPad, Padding};
use crate::layout::LayoutContext;
use crate::yard::{ArcYard, Yard};

impl Padding for ArcYard {
	fn pad(self, size: i32) -> ArcYard {
		PadYard::new(size, self)
	}

	fn pad_cols(self, cols: i32) -> ArcYard {
		PadYard::pad_cols(cols, self)
	}
}

struct PadYard {
	id: i32,
	left_cols: i32,
	right_cols: i32,
	top_rows: i32,
	bottom_rows: i32,
	yard: ArcYard,
}

impl PadYard {
	fn pad_cols(cols: i32, yard: ArcYard) -> ArcYard {
		Arc::new(PadYard {
			id: rand::random(),
			left_cols: cols,
			right_cols: cols,
			top_rows: 0,
			bottom_rows: 0,
			yard,
		})
	}
	fn new(size: i32, yard: ArcYard) -> ArcYard {
		let cols = size * 2;
		let rows = size;
		Arc::new(PadYard {
			id: rand::random(),
			left_cols: cols,
			right_cols: cols,
			top_rows: rows,
			bottom_rows: rows,
			yard,
		})
	}
}

impl Yard for PadYard {
	fn id(&self) -> i32 { self.id }
	fn type_desc(&self) -> &'static str { "Pad" }

	fn layout(&self, ctx: &mut LayoutContext) -> usize {
		let (edge_index, edge_bounds) = ctx.edge_bounds();
		let alt_bounds = edge_bounds.pad(self.left_cols, self.right_cols, self.top_rows, self.bottom_rows);
		let alt_index = ctx.push_bounds(&alt_bounds);
		let mut alt_ctx = ctx.with_index(alt_index);
		let core_index = self.yard.layout(&mut alt_ctx);
		if core_index == alt_index {
			edge_index
		} else {
			let core_bounds = ctx.bounds(core_index);
			let final_bounds = edge_bounds.with_z(core_bounds.z);
			let final_index = ctx.push_bounds(&final_bounds);
			final_index
		}
	}

	fn render(&self, _bounds: &Bounds, _focus_id: i32, _pad: &mut dyn DrawPad) -> Option<Vec<(ArcYard, Option<i32>)>> {
		Some(vec![(self.yard.clone(), None)])
	}
}
