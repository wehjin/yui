use std::sync::{Arc, RwLock};

use crate::yard::{ArcTouch, ArcYard, Yard, YardOption};
use crate::yard;
use crate::yui::{Cling, Focus, FocusType, render_submit, RenderContext};
use crate::yui::layout::LayoutContext;
use crate::palette::{FillColor, StrokeColor};

pub fn button<S: AsRef<str>>(text: S, on_click: impl Fn(i32) + Send + Sync + 'static) -> ArcYard {
	let id = rand::random();
	Arc::new(ButtonYard {
		id,
		label_yard: yard::label(
			&text.as_ref().to_uppercase(),
			StrokeColor::EnabledOnBackground,
			Cling::Center,
		),
		fill_yard: yard::fill(FillColor::BackgroundWithFocus),
		is_pressed: Arc::new(RwLock::new(false)),
		on_click: Arc::new(move || on_click(id)),
	})
}

struct ButtonYard {
	id: i32,
	label_yard: ArcYard,
	fill_yard: ArcYard,
	is_pressed: Arc<RwLock<bool>>,
	on_click: ArcTouch,
}

impl Yard for ButtonYard {
	fn id(&self) -> i32 { self.id }

	fn update(&self, _option: YardOption) {}

	fn layout(&self, ctx: &mut LayoutContext) -> usize {
		let (edge_index, edge_bounds) = ctx.edge_bounds();
		self.fill_yard.layout(ctx);
		self.label_yard.layout(ctx);
		let is_pressed = self.is_pressed.clone();
		let on_click = self.on_click.to_owned();
		ctx.add_focus(Focus {
			yard_id: self.id(),
			focus_type: FocusType::Submit,
			bounds: edge_bounds,
			action_block: Arc::new(move |ctx| render_submit(&is_pressed, ctx, &on_click)),
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

