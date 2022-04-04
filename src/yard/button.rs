use std::sync::{Arc, RwLock};

use crate::{DrawPad, Link, SenderLink, SyncLink, yard};
use crate::bounds::Bounds;
use crate::layout::LayoutContext;
use crate::palette::{FillGrade, StrokeColor};
use crate::yard::{ArcTouch, ArcYard, Yard, YardOption};
use crate::yui::{Cling, Focus, FocusType, render_submit};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Priority {
	None,
	Default,
}

pub enum ButtonState {
	Enabled(Priority, SenderLink<i32>),
	Disabled,
}

impl ButtonState {
	pub fn is_enabled(&self) -> bool {
		match self {
			ButtonState::Enabled(_, _) => true,
			ButtonState::Disabled => false,
		}
	}
	pub fn enabled(click: SenderLink<i32>) -> Self { ButtonState::Enabled(Priority::None, click) }
	pub fn default(click: SenderLink<i32>) -> Self { ButtonState::Enabled(Priority::Default, click) }
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
			let sync_click: SyncLink<i32> = click.into();
			(priority, Some(Arc::new(move || sync_click.send(id)) as ArcTouch))
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
	fn id(&self) -> i32 { self.id }

	fn update(&self, _option: YardOption) {}

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

	fn render(&self, bounds: &Bounds, focus_id: i32, pad: &mut dyn DrawPad) -> Option<Vec<(ArcYard, Option<i32>)>> {
		let fill_grade = if focus_id == self.id {
			let is_pressed = { *self.is_pressed.read().expect("read is_pressed") };
			if is_pressed { FillGrade::Press } else { FillGrade::Focus }
		} else {
			FillGrade::Plain
		};
		pad.grade(bounds, fill_grade);
		Some(vec![(self.label_yard.clone(), None)])
	}
}

