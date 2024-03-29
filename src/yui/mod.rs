use std::{fmt, thread};
use std::fmt::{Debug, Formatter};
use std::sync::Arc;

pub use stringedit::{Action as StringEditAction, StringEdit, Validity as ValidIf};

pub use multi_layout::*;

use crate::core::bounds::Bounds;
use crate::yard::ArcYard;

pub mod layout;
pub mod pad;
pub mod place;
pub mod confine;
mod multi_layout;
pub mod prelude;

pub enum FocusAction {
	Go,
	Change(char),
}

#[derive(Clone)]
pub struct Focus {
	pub yard_id: i32,
	pub focus_type: FocusType,
	pub bounds: Bounds,
	pub priority: u32,
	pub action_block: Arc<dyn Fn(&FocusActionContext) + Send + Sync>,
}

impl Debug for Focus {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		f.debug_struct("Focus")
			.field("yard", &self.yard_id)
			.field("priority", &self.priority)
			.field("bounds", &self.bounds)
			.finish()
	}
}

impl Focus {
	pub fn expand_seam(&self, z: i32, depth: i32) -> Self {
		let mut focus = self.clone();
		focus.bounds = self.bounds.expand_seam(z, depth);
		focus
	}
	pub fn shift_seam(&self, z: i32, left: i32, top: i32) -> Self {
		let mut focus = self.clone();
		focus.bounds = self.bounds.shift_seam(z, left, top);
		focus
	}

	pub fn is_in_range(&self, focus_max: i32) -> bool {
		self.bounds.z <= focus_max
	}

	pub fn insert_char(&self, char: char, refresh: impl Fn() + Send + 'static) {
		match self.focus_type {
			FocusType::Submit | FocusType::CompositeSubmit(_) => {}
			FocusType::Edit(_) => {
				let action_block = self.action_block.clone();
				thread::Builder::new().name("Focus::insert_char".to_string()).spawn(move || {
					let ctx = FocusActionContext {
						action: FocusAction::Change(char),
						refresh: Box::new(refresh),
					};
					action_block(&ctx);
				}).expect("spawn");
			}
		};
	}

	pub fn insert_space(&self, refresh: impl Fn() + Send + 'static) {
		match self.focus_type {
			FocusType::Edit(_) => {
				let action_block = self.action_block.clone();
				thread::Builder::new().name("insert_space Edit".to_string()).spawn(move || {
					let ctx = FocusActionContext {
						action: FocusAction::Change(' '),
						refresh: Box::new(refresh),
					};
					action_block(&ctx);
				}).expect("spawn");
			}
			FocusType::Submit | FocusType::CompositeSubmit(_) => {
				let action_block = self.action_block.clone();
				thread::Builder::new().name("insert_space Submit".to_string()).spawn(move || {
					let ctx = FocusActionContext {
						action: FocusAction::Go,
						refresh: Box::new(refresh),
					};
					action_block(&ctx);
				}).expect("spawn");
			}
		};
	}
}

pub struct FocusActionContext {
	pub action: FocusAction,
	pub refresh: Box<dyn Fn()>,
}

#[derive(Clone)]
pub enum FocusType {
	Submit,
	Edit(Arc<dyn Fn(FocusMotion) -> FocusMotionFuture + Send + Sync>),
	CompositeSubmit(Arc<dyn Fn(FocusMotion) -> FocusMotionFuture + Send + Sync>),
}

impl fmt::Debug for FocusType {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		match self {
			FocusType::Submit => f.write_str("FocusType::Submit"),
			FocusType::Edit(_) => f.write_str("FocusType::Edit(Arc<dyn Fn(&FocusMotion) -> AfterMotion>)"),
			FocusType::CompositeSubmit(_) => f.write_str("FocusType::CompositeSubmit(Arc<dyn Fn(&FocusMotion) -> AfterMotion>)"),
		}
	}
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum FocusMotion {
	Left,
	Right,
	Up,
	Down,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum FocusMotionFuture {
	Default,
	Skip,
}

pub trait Padding {
	fn pad(self, size: i32) -> ArcYard;
	fn pad_cols(self, cols: i32) -> ArcYard;
}

pub trait Confine {
	fn confine_height(self, height: i32, cling: Cling) -> ArcYard;
	fn confine_width(self, width: i32, cling: Cling) -> ArcYard;
	fn confine(self, width: i32, height: i32, cling: Cling) -> ArcYard;
}

pub trait Fade {
	fn fade(self, indents: (i32, i32), fore_yard: ArcYard) -> ArcYard;
}

pub trait Before {
	fn before(self, yard: ArcYard) -> ArcYard;
}

pub trait Pack {
	fn pack_top(self, rows: i32, top_yard: ArcYard) -> ArcYard;
	fn pack_bottom(self, rows: i32, bottom_yard: ArcYard) -> ArcYard;
	fn pack_left(self, cols: i32, left_yard: ArcYard) -> ArcYard;
	fn pack_right(self, cols: i32, right_yard: ArcYard) -> ArcYard;
}

pub trait Place {
	fn place_center(self, width: i32) -> ArcYard;
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Cling {
	Custom { x: f32, y: f32 },
	Center,
	Top,
	Bottom,
	Left,
	LeftTop,
	LeftBottom,
	Right,
	RightTop,
	RightBottom,
}

impl Cling {
	pub fn x(&self) -> f32 {
		match self {
			Cling::Custom { x, .. } => x.to_owned(),
			Cling::Left | Cling::LeftTop | Cling::LeftBottom => 0.0,
			Cling::Center | Cling::Top | Cling::Bottom => 0.5,
			Cling::Right | Cling::RightTop | Cling::RightBottom => 1.0,
		}
	}
	pub fn y(&self) -> f32 {
		match self {
			Cling::Top | Cling::RightTop | Cling::LeftTop => 0.0,
			Cling::Center | Cling::Left | Cling::Right => 0.5,
			Cling::Bottom | Cling::LeftBottom | Cling::RightBottom => 1.0,
			Cling::Custom { y, .. } => y.to_owned(),
		}
	}
}

impl From<Cling> for (f32, f32) {
	fn from(cling: Cling) -> Self {
		(cling.x(), cling.y())
	}
}
