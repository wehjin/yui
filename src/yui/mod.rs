use std::{fmt, thread};
use std::fmt::{Debug, Formatter};
use std::ops::Deref;
use std::sync::{Arc, RwLock};
use std::time::Duration;

pub use stringedit::*;

pub use multi_layout::*;

use crate::yard::{ArcTouch, ArcYard};

use self::bounds::Bounds;
use self::palette::{FillColor, StrokeColor};

pub mod bounds;
pub mod layout;
pub mod pad;
pub mod before;
pub mod palette;
pub mod pack;
pub mod place;
pub mod confine;
pub mod tabbar;
mod multi_layout;

pub enum FocusAction {
	Go,
	Change(char),
}

#[derive(Clone)]
pub struct Focus {
	pub yard_id: i32,
	pub focus_type: FocusType,
	pub bounds: Bounds,
	pub action_block: Arc<dyn Fn(&FocusActionContext) + Send + Sync>,
}

impl Focus {
	pub fn is_in_range(&self, focus_max: i32) -> bool {
		self.bounds.z <= focus_max
	}

	pub fn insert_char(&self, char: char, refresh: impl Fn() + Send + 'static) {
		match self.focus_type {
			FocusType::Submit => {}
			FocusType::Edit(_) => {
				let action_block = self.action_block.clone();
				thread::spawn(move || {
					let ctx = FocusActionContext {
						action: FocusAction::Change(char),
						refresh: Box::new(refresh),
					};
					action_block(&ctx);
				});
			}
		};
	}

	pub fn insert_space(&self, refresh: impl Fn() + Send + 'static) {
		match self.focus_type {
			FocusType::Edit(_) => {
				let action_block = self.action_block.clone();
				thread::spawn(move || {
					let ctx = FocusActionContext {
						action: FocusAction::Change(' '),
						refresh: Box::new(refresh),
					};
					action_block(&ctx);
				});
			}
			FocusType::Submit => {
				let action_block = self.action_block.clone();
				thread::spawn(move || {
					let ctx = FocusActionContext {
						action: FocusAction::Go,
						refresh: Box::new(refresh),
					};
					action_block(&ctx);
				});
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
}

impl fmt::Debug for FocusType {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		match self {
			FocusType::Submit => f.write_str("FocusType::Submit"),
			FocusType::Edit(_) => f.write_str("FocusType::Edit(Arc<dyn Fn(&FocusMotion) -> AfterMotion>)"),
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


pub fn render_submit(is_pressed: &Arc<RwLock<bool>>, ctx: &FocusActionContext, touch: &ArcTouch) -> () {
	{
		*is_pressed.write().unwrap() = true;
	}
	ctx.refresh.deref()();
	thread::sleep(Duration::from_millis(100));
	{
		*is_pressed.write().unwrap() = false;
	}
	ctx.refresh.deref()();
	touch.deref()();
	ctx.refresh.deref()();
}

pub trait RenderContext {
	fn focus_id(&self) -> i32;
	fn spot(&self) -> (i32, i32);
	fn yard_bounds(&self, yard_id: i32) -> Bounds;
	fn set_fill(&self, color: FillColor, z: i32);
	fn set_glyph(&self, glyph: char, color: StrokeColor, z: i32);
	fn set_dark(&self, z: i32);
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
	fn x(&self) -> f32 {
		match self {
			Cling::Custom { x, .. } => x.to_owned(),
			Cling::Left | Cling::LeftTop | Cling::LeftBottom => 0.0,
			Cling::Center | Cling::Top | Cling::Bottom => 0.5,
			Cling::Right | Cling::RightTop | Cling::RightBottom => 1.0,
		}
	}
	fn y(&self) -> f32 {
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
