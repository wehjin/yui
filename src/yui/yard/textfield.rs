use std::sync::Arc;

use crate::yui::{ArcYard, Before, Cling, MultiLayout, Padding, RenderContext, Yard, yard, YardOption};
use crate::yui::fill::fill_yard;
use crate::yui::glyph::glyph_yard;
use crate::yui::layout::LayoutContext;
use crate::yui::palette::{FillColor, StrokeColor};

pub fn textfield(label: &str) -> ArcYard {
	let yard = TextfieldYard {
		id: rand::random(),
		head: yard::label(label, StrokeColor::CommentOnBackground, Cling::Left).pad_cols(1),
		body: yard::label("Value", StrokeColor::BodyOnBackground, Cling::Left).pad_cols(1),
		foot: glyph_yard(StrokeColor::CommentOnBackground, || '_'),
	};
	let arc_yard = Arc::new(yard) as ArcYard;
	arc_yard.before(fill_yard(FillColor::BackgroundWithFocus))
}

struct TextfieldYard {
	id: i32,
	head: ArcYard,
	body: ArcYard,
	foot: ArcYard,
}

impl Yard for TextfieldYard {
	fn id(&self) -> i32 { self.id }

	fn update(&self, option: YardOption) {
		self.head.update(option)
	}

	fn layout(&self, ctx: &mut LayoutContext) -> usize {
		let (_edge_index, edge_bounds) = ctx.edge_bounds();
		let (head_bounds, lower_bounds) = edge_bounds.split_from_top(1);
		let (body_bounds, foot_bounds) = lower_bounds.split_from_bottom(1);
		let mut multi_layout = MultiLayout::new(ctx);
		multi_layout.layout(&self.head, &head_bounds);
		multi_layout.layout(&self.body, &body_bounds);
		multi_layout.layout(&self.foot, &foot_bounds);
		multi_layout.finish()
	}

	fn render(&self, ctx: &dyn RenderContext) {
		self.head.render(ctx);
		self.body.render(ctx);
		self.foot.render(ctx);
	}
}