use std::sync::{Arc, RwLock};

use crate::palette::{FillGrade, StrokeColor};
use crate::yard::{ArcTouch, ArcYard, Yard, YardOption};
use crate::yard;
use crate::yui::{Cling, Focus, FocusType, render_submit, RenderContext};
use crate::yui::layout::LayoutContext;

pub fn button_enabled<S: AsRef<str> + std::fmt::Display>(text: S, on_click: impl Fn(i32) + Send + Sync + 'static) -> ArcYard {
	button(text, ActiveState::Enabled(Box::new(on_click)))
}

pub fn button_disabled<S: AsRef<str> + std::fmt::Display>(text: S) -> ArcYard {
	button(text, ActiveState::Disabled)
}

struct ButtonYard {
	id: i32,
	label_yard: ArcYard,
	is_pressed: Arc<RwLock<bool>>,
	click_option: Option<ArcTouch>,
}

pub fn button<S: AsRef<str> + std::fmt::Display>(text: S, active_state: ActiveState) -> ArcYard {
	let id = rand::random();
	let label_yard = yard::label(
		&text.as_ref().to_uppercase(),
		match &active_state {
			ActiveState::Enabled(_) => StrokeColor::EnabledOnBackground,
			ActiveState::Disabled => StrokeColor::CommentOnBackground,
		},
		Cling::Center,
	);
	let is_pressed = Arc::new(RwLock::new(false));
	let click_option: Option<ArcTouch> = match active_state {
		ActiveState::Enabled(on_click) => Some(Arc::new(move || on_click(id))),
		ActiveState::Disabled => None,
	};
	Arc::new(ButtonYard { id, label_yard, is_pressed, click_option })
}

pub enum ActiveState {
	Enabled(Box<dyn Fn(i32) + Send + Sync + 'static>),
	Disabled,
}

impl Yard for ButtonYard {
	fn render(&self, ctx: &dyn RenderContext) {
		let (row, col) = ctx.spot();
		let bounds = ctx.yard_bounds(self.id);
		if bounds.intersects(row, col) {
			let focus_id = ctx.focus_id();
			let fill_grade = if focus_id == self.id {
				let is_pressed = { *self.is_pressed.read().unwrap() };
				if is_pressed { FillGrade::Press } else { FillGrade::Focus }
			} else {
				FillGrade::Plain
			};
			ctx.set_fill_grade(fill_grade, bounds.z)
		}
		self.label_yard.render(ctx);
	}

	fn layout(&self, ctx: &mut LayoutContext) -> usize {
		let (edge_index, edge_bounds) = ctx.edge_bounds();
		self.label_yard.layout(ctx);
		match &self.click_option {
			None => {}
			Some(on_click) => {
				let on_click = on_click.to_owned();
				let is_pressed = self.is_pressed.clone();
				ctx.add_focus(Focus {
					yard_id: self.id,
					focus_type: FocusType::Submit,
					bounds: edge_bounds,
					action_block: Arc::new(move |ctx| render_submit(&is_pressed, ctx, &on_click)),
				});
			}
		}
		ctx.set_yard_bounds(self.id, edge_index);
		edge_index
	}

	fn update(&self, _option: YardOption) {}

	fn id(&self) -> i32 { self.id }
}

