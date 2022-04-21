use std::sync::Arc;

use crate::{Bounds, DrawPad, Fade, MultiLayout};
use crate::layout::LayoutContext;
use crate::yard::{ArcYard, Yard};

pub fn fade(indents: (i32, i32), rear_yard: ArcYard, fore_yard: ArcYard) -> ArcYard {
	Arc::new(FadeYard {
		id: rand::random(),
		indents,
		rear_yard,
		fore_yard,
	})
}

struct FadeYard {
	id: i32,
	indents: (i32, i32),
	rear_yard: ArcYard,
	fore_yard: ArcYard,
}

impl Fade for ArcYard {
	fn fade(self, indents: (i32, i32), fore_yard: ArcYard) -> ArcYard {
		fade(indents, self, fore_yard)
	}
}

impl Yard for FadeYard {
	fn id(&self) -> i32 {
		self.id
	}

	fn layout(&self, ctx: &mut LayoutContext) -> usize {
		let (_bounds_id, bounds) = ctx.edge_bounds();
		let mut multi_layout = MultiLayout::new(ctx);
		multi_layout.layout(&self.rear_yard, &bounds);

		let (cols, rows) = self.indents;
		let indent_bounds = bounds.pad(cols, cols, rows, rows);
		let rear_near_z = multi_layout.near_z();
		let fore_z = rear_near_z - 1;
		let fore_bounds = indent_bounds.with_z(fore_z);
		multi_layout.layout(&self.fore_yard, &fore_bounds);

		// TODO: Record rear_near_z so we can draw the darkened area a the proper z.
		let end_index = multi_layout.finish();
		ctx.set_yard_bounds(self.id(), end_index);
		ctx.set_focus_max(fore_z);
		end_index
	}

	fn render(&self, bounds: &Bounds, _focus_id: i32, pad: &mut dyn DrawPad) -> Option<Vec<(ArcYard, Option<i32>)>> {
		let (cols, rows) = self.indents;
		let inside = bounds.pad(cols, cols, rows, rows);
		pad.dark(bounds, &inside);
		Some(vec![(self.fore_yard.clone(), None), (self.rear_yard.clone(), None)])
	}
}

#[cfg(test)]
mod tests {
	use crate::{FillColor, FillGrade, layout, render, yard};
	use crate::FillColor::{Background, Primary};
	use crate::yui::layout::ActiveFocus;

	#[test]
	fn layout_render() {
		let back = yard::fill(FillColor::Background, FillGrade::Plain);
		let fore = yard::fill(FillColor::Primary, FillGrade::Plain);
		let yard = yard::fade((1, 1), back, fore);
		let (max_x, max_y) = (3, 3);
		let layout = layout::run(max_y, max_x, &yard, &ActiveFocus::default());
		let spot_table = render::run(&yard, layout.max_x, layout.max_y, layout.bounds_hold.clone(), 0);
		let fronts = spot_table.to_fronts();
		let computed = fronts.iter().flatten()
			.map(|front| (front.fill_color, front.dark))
			.collect::<Vec<_>>();
		assert_eq!(computed, vec![
			(Background, true), (Background, true), (Background, true),
			(Background, true), (Primary, false), (Background, true),
			(Background, true), (Background, true), (Background, true),
		])
	}
}