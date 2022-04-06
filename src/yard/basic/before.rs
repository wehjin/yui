use std::sync::Arc;

use crate::{Before, Bounds, DrawPad, MultiLayout};
use crate::layout::LayoutContext;
use crate::yard::{ArcYard, Yard, YardOption};

impl Before for ArcYard {
	fn before(self, far_yard: ArcYard) -> ArcYard {
		BeforeYard::new(self, far_yard)
	}
}

struct BeforeYard {
	id: i32,
	near_yard: ArcYard,
	far_yard: ArcYard,
}

impl BeforeYard {
	fn new(near_yard: ArcYard, far_yard: ArcYard) -> ArcYard {
		Arc::new(BeforeYard {
			id: rand::random(),
			near_yard,
			far_yard,
		})
	}
}

impl Yard for BeforeYard {
	fn id(&self) -> i32 { self.id }
	fn type_desc(&self) -> &'static str { "Before" }

	fn update(&self, _option: YardOption) {}

	fn layout(&self, ctx: &mut LayoutContext) -> usize {
		let (_edge_index, edge_bounds) = ctx.edge_bounds();
		let mut multi_layout = MultiLayout::new(ctx);
		multi_layout.layout(&self.far_yard, &edge_bounds);
		multi_layout.layout(&self.near_yard, &edge_bounds.with_z(multi_layout.near_z() - 1));
		let final_index = multi_layout.finish();
		ctx.set_yard_bounds(self.id, final_index);
		final_index
	}

	fn render(&self, _bounds: &Bounds, _focus_id: i32, _pad: &mut dyn DrawPad) -> Option<Vec<(ArcYard, Option<i32>)>> {
		Some(vec![
			(self.far_yard.clone(), None),
			(self.near_yard.clone(), None),
		])
	}
}

#[cfg(test)]
mod tests {
	use crate::{Before, FillColor, FillGrade, layout, render, SenderLink, yard};
	use crate::yui::layout::ActiveFocus;

	#[test]
	fn layout_render() {
		let white = yard::fill(FillColor::Background, FillGrade::Plain);
		let black = yard::fill(FillColor::Primary, FillGrade::Plain);
		let yard = black.before(white);
		println!("START_ID: {}", yard.id());

		let (max_x, max_y) = (2, 1);
		let layout = layout::run(max_x, max_y, &yard, &SenderLink::ignore(), &ActiveFocus::default());
		let draw_pad = render::run(&yard, layout.max_x, layout.max_y, layout.bounds_hold.clone(), 0);
		let fronts = draw_pad.to_fronts();
		let fills = fronts.iter().flatten().map(|front| front.fill_color).collect::<Vec<_>>();
		assert_eq!(fills, vec![FillColor::Primary, FillColor::Primary]);
	}
}