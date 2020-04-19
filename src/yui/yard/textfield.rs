use std::sync::Arc;

use crate::yui::{ArcYard, Cling, RenderContext, Yard, yard, YardOption};
use crate::yui::layout::LayoutContext;
use crate::yui::palette::StrokeColor;

pub fn textfield(label: &str) -> ArcYard {
	let yard = TextfieldYard {
		id: rand::random(),
		label: yard::label(label, StrokeColor::BodyOnBackground, Cling::Left),
	};
	Arc::new(yard)
}

struct TextfieldYard {
	id: i32,
	label: ArcYard,
}

impl Yard for TextfieldYard {
	fn id(&self) -> i32 { self.id() }

	fn update(&self, option: YardOption) {
		self.label.update(option)
	}

	fn layout(&self, ctx: &mut LayoutContext) -> usize {
		let (_edge_index, edge_bounds) = ctx.edge_bounds();
		let (label_edge_bounds, _) = edge_bounds.split_from_top(1);
		let label_edge_index = ctx.push_bounds(&label_edge_bounds);
		self.label.layout(&mut ctx.with_index(label_edge_index))
	}

	fn render(&self, ctx: &dyn RenderContext) {
		self.label.render(ctx)
	}
}