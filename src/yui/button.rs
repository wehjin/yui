use std::sync::{Arc, RwLock};

use crate::yard::{ArcYard, Yard, YardOption};
use crate::yard;
use crate::yui::{Cling, Focus, FocusType, render_submit, RenderContext};
use crate::yui::layout::LayoutContext;
use crate::yui::palette::{FillColor, StrokeColor};

pub fn button_yard(text: &str) -> ArcYard {
	ButtonYard::new(text)
}

struct ButtonYard {
	id: i32,
	label_yard: ArcYard,
	fill_yard: ArcYard,
	is_pressed: Arc<RwLock<bool>>,
}

impl ButtonYard {
	fn new(text: &str) -> ArcYard {
		Arc::new(ButtonYard {
			id: rand::random(),
			label_yard: yard::label(
				&text.to_uppercase(),
				StrokeColor::EnabledOnBackground,
				Cling::CenterMiddle,
			),
			fill_yard: yard::fill(FillColor::BackgroundWithFocus),
			is_pressed: Arc::new(RwLock::new(false)),
		})
	}
}

impl Yard for ButtonYard {
	fn id(&self) -> i32 {
		self.id
	}

	fn update(&self, _option: YardOption) {}

	fn layout(&self, ctx: &mut LayoutContext) -> usize {
		let (edge_index, edge_bounds) = ctx.edge_bounds();
		self.fill_yard.layout(ctx);
		self.label_yard.layout(ctx);
		let is_pressed = self.is_pressed.clone();
		ctx.add_focus(Focus {
			yard_id: self.id(),
			focus_type: FocusType::Submit,
			bounds: edge_bounds,
			action_block: Arc::new(move |ctx| render_submit(&is_pressed, ctx)),
		});
		edge_index
	}

	fn render(&self, ctx: &dyn RenderContext) {
		let focus_id = ctx.focus_id();
		let fill_color = if focus_id == self.id {
			let is_pressed = { *self.is_pressed.read().unwrap() };
			if is_pressed {
				FillColor::BackgroundWithPress
			} else {
				FillColor::BackgroundWithFocus
			}
		} else {
			FillColor::Background
		};
		self.fill_yard.update(YardOption::FillColor(fill_color));
		self.fill_yard.render(ctx);
		self.label_yard.render(ctx);
	}
}

