use std::sync::{Arc, RwLock};

use crate::palette::{FillGrade, StrokeColor};
use crate::yard::{ArcTouch, ArcYard, Yard, YardOption};
use crate::yard;
use crate::yui::{Cling, Focus, FocusType, render_submit, RenderContext};
use crate::yui::layout::LayoutContext;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Priority {
	None,
	Default,
}

pub enum ButtonState {
	Enabled(Priority, Box<dyn Fn(i32) + Send + Sync + 'static>),
	Disabled,
}

impl ButtonState {
	pub fn is_enabled(&self) -> bool {
		match self {
			ButtonState::Enabled(_, _) => true,
			ButtonState::Disabled => false,
		}
	}
	pub fn enabled(click: impl Fn(i32) + Send + Sync + 'static) -> Self { ButtonState::Enabled(Priority::None, Box::new(click)) }
	pub fn default(click: impl Fn(i32) + Send + Sync + 'static) -> Self { ButtonState::Enabled(Priority::Default, Box::new(click)) }
	pub fn disabled() -> Self { ButtonState::Disabled }
}

pub fn button<S: AsRef<str> + std::fmt::Display>(text: S, state: ButtonState) -> ArcYard {
	let id = rand::random();
	let stroke_color = if state.is_enabled() { StrokeColor::EnabledOnBackground } else { StrokeColor::CommentOnBackground };
	let uppercase_text = &text.as_ref().to_uppercase();
	let label_yard = yard::label(uppercase_text, stroke_color, Cling::Center);
	let is_pressed = Arc::new(RwLock::new(false));
	let (priority, click_option) = match state {
		ButtonState::Enabled(priority, click) => {
			let priority = match priority {
				Priority::None => 0,
				Priority::Default => 1000,
			};
			(priority, Some(Arc::new(move || click(id)) as ArcTouch))
		}
		ButtonState::Disabled => (0, None),
	};
	Arc::new(ButtonYard { id, label_yard, is_pressed, click_option, priority })
}

struct ButtonYard {
	id: i32,
	label_yard: ArcYard,
	is_pressed: Arc<RwLock<bool>>,
	click_option: Option<ArcTouch>,
	priority: u32,
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
					priority: self.priority,
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

