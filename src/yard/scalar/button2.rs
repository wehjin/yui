use std::sync::Arc;
use std::thread;
use std::time::Duration;

use rand::random;

use crate::{DrawPad, Link, SyncLink, Trigger, yard};
use crate::bounds::Bounds;
use crate::layout::LayoutContext;
use crate::palette::{FillGrade, StrokeColor};
use crate::yard::{ArcYard, Yard, YardOption};
use crate::yui::{Cling, Focus, FocusType};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Priority {
	None,
	Default,
}

#[derive(Debug, Clone)]
pub enum SubmitAffordance {
	Disabled,
	Enabled { press_link: SyncLink<i32>, priority: Priority },
	Pressed { press_link: SyncLink<i32>, priority: Priority },
}

impl SubmitAffordance {
	pub fn enabled(press_link: SyncLink<i32>) -> Self {
		SubmitAffordance::Enabled { press_link, priority: Priority::None }
	}
}

#[derive(Debug, Copy, Clone)]
pub enum ButtonAction {
	Press,
	Release,
}

#[derive(Debug, Clone)]
pub struct ButtonModel {
	pub id: i32,
	pub label: String,
	pub release_trigger: Trigger,
	pub affordance: SubmitAffordance,
}

impl ButtonModel {
	pub fn disabled(label: &str, release_trigger: Trigger) -> Self {
		let id = random();
		let label = label.into();
		let affordance = SubmitAffordance::Disabled;
		ButtonModel { id, label, release_trigger, affordance }
	}
	pub fn enabled(label: &str, release_trigger: Trigger, press_link: SyncLink<i32>) -> Self {
		let id = random();
		let label = label.to_string();
		let affordance = SubmitAffordance::enabled(press_link);
		ButtonModel { id, label, release_trigger, affordance }
	}
	pub fn set_label(&self, label: &str) -> Self {
		ButtonModel { label: label.to_string(), ..self.clone() }
	}
	pub fn enable(&self, label: &str, press_link: SyncLink<i32>) -> Self {
		let mut button = self.clone();
		button.label = label.to_string();
		button.affordance = SubmitAffordance::enabled(press_link);
		button
	}
	pub fn disable(&self, label: &str) -> Self {
		let mut button = self.clone();
		button.label = label.to_string();
		button.affordance = SubmitAffordance::Disabled;
		button
	}
	pub fn update(&self, action: ButtonAction) -> Self {
		match action {
			ButtonAction::Press => self.press(),
			ButtonAction::Release => self.release(),
		}
	}
	fn press(&self) -> Self {
		let (new_affordance, submit_trigger) = match &self.affordance {
			SubmitAffordance::Disabled => (SubmitAffordance::Disabled, None),
			SubmitAffordance::Enabled { press_link: press, priority, .. } => (SubmitAffordance::Pressed { press_link: press.clone(), priority: priority.clone() }, Some(self.release_trigger.clone())),
			SubmitAffordance::Pressed { press_link: press, priority } => (SubmitAffordance::Pressed { press_link: press.clone(), priority: priority.clone() }, None),
		};
		if let Some(trigger) = submit_trigger {
			thread::spawn(move || {
				thread::sleep(Duration::from_millis(300));
				trigger.send(());
			});
		}
		ButtonModel { affordance: new_affordance, ..self.clone() }
	}
	fn release(&self) -> Self {
		let material = match &self.affordance {
			SubmitAffordance::Disabled => SubmitAffordance::Disabled,
			SubmitAffordance::Enabled { press_link: press, priority, .. } => SubmitAffordance::Enabled { press_link: press.clone(), priority: priority.clone() },
			SubmitAffordance::Pressed { press_link: press, priority } => SubmitAffordance::Enabled { press_link: press.clone(), priority: priority.clone() }
		};
		ButtonModel { affordance: material, ..self.clone() }
	}
}

pub fn button2(button: &ButtonModel) -> ArcYard {
	let label_yard = yard::label(
		&button.label.to_uppercase(),
		match &button.affordance {
			SubmitAffordance::Disabled => StrokeColor::CommentOnBackground,
			SubmitAffordance::Enabled { .. } => StrokeColor::EnabledOnBackground,
			SubmitAffordance::Pressed { .. } => StrokeColor::EnabledOnBackground,
		},
		Cling::Center,
	);
	let material = button.affordance.clone();
	Arc::new(ButtonYard { id: button.id, label_yard, material })
}

fn focus_priority(priority: &Priority) -> u32 {
	match priority {
		Priority::None => 0,
		Priority::Default => 1000,
	}
}

struct ButtonYard {
	id: i32,
	label_yard: ArcYard,
	material: SubmitAffordance,
}

impl Yard for ButtonYard {
	fn id(&self) -> i32 { self.id }

	fn update(&self, _option: YardOption) {}

	fn layout(&self, ctx: &mut LayoutContext) -> usize {
		let (edge_index, edge_bounds) = ctx.edge_bounds();
		self.label_yard.layout(ctx);
		match &self.material {
			SubmitAffordance::Disabled => {}
			SubmitAffordance::Enabled { press_link: press, priority } => {
				let id = self.id;
				let press = press.clone();
				ctx.add_focus(Focus {
					yard_id: self.id,
					focus_type: FocusType::Submit,
					bounds: edge_bounds,
					priority: focus_priority(&priority),
					action_block: Arc::new(move |_ctx| press.send(id)),
				})
			}
			SubmitAffordance::Pressed { priority, .. } => {
				// Register focus to keep its place as active focus, but ignore inputs.
				ctx.add_focus(Focus {
					yard_id: self.id,
					focus_type: FocusType::Submit,
					bounds: edge_bounds,
					priority: focus_priority(&priority),
					action_block: Arc::new(move |_ctx| {}),
				})
			}
		}
		ctx.set_yard_bounds(self.id, edge_index);
		edge_index
	}

	fn render(&self, bounds: &Bounds, focus_id: i32, pad: &mut dyn DrawPad) -> Option<Vec<(ArcYard, Option<i32>)>> {
		let fill_grade = if focus_id == self.id {
			match self.material {
				SubmitAffordance::Disabled => panic!("disable button is focus"),
				SubmitAffordance::Enabled { .. } => FillGrade::Focus,
				SubmitAffordance::Pressed { .. } => FillGrade::Press,
			}
		} else {
			FillGrade::Plain
		};
		pad.grade(bounds, fill_grade);
		Some(vec![(self.label_yard.clone(), None)])
	}
}
