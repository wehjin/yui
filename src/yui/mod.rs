use std::{fmt, thread};
use std::error::Error;
use std::fmt::{Debug, Formatter};
use std::ops::Deref;
use std::sync::{Arc, RwLock};
use std::time::Duration;

pub use stringedit::{Action as StringEditAction, StringEdit, Validity as ValidIf};

pub use multi_layout::*;

use crate::{app, Spark};
use crate::palette::{FillColor, FillGrade, StrokeColor};
use crate::yard::{ArcTouch, ArcYard};

use self::bounds::Bounds;

pub mod bounds;
pub mod layout;
pub mod pad;
pub mod before;
pub mod pack;
pub mod place;
pub mod confine;
mod multi_layout;
pub mod stories;
pub mod prelude;

pub fn main<T>(spark: T) -> Result<(), Box<dyn Error>> where T: Spark + Sync + Send + 'static {
	//! Activate the main yui interaction.
	app::run(spark, None)
}

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

impl Focus {
	pub fn is_in_range(&self, focus_max: i32) -> bool {
		self.bounds.z <= focus_max
	}

	pub fn insert_char(&self, char: char, refresh: impl Fn() + Send + 'static) {
		match self.focus_type {
			FocusType::Submit | FocusType::CompositeSubmit(_) => {}
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
			FocusType::Submit | FocusType::CompositeSubmit(_) => {
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


pub(crate) fn render_submit(is_pressed: &Arc<RwLock<bool>>, ctx: &FocusActionContext, touch: &ArcTouch) -> () {
	{
		*is_pressed.write().unwrap() = true;
	}
	ctx.refresh.deref()();
	{
		// For longer yard::lists, 100 is too fast for the pressed state to render
		// consistently when the cursor is at the bottom of the list.
		let millis = if cfg!(debug_assertions) {
			200
		} else {
			100
		};
		thread::sleep(Duration::from_millis(millis));
	}
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
	fn set_fill_grade(&self, fill_grade: FillGrade, z: i32);
	fn set_glyph(&self, glyph: String, color: StrokeColor, z: i32);
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

pub struct FocusIdRenderContext<'a> {
	pub parent: &'a dyn RenderContext,
	pub focus_id: i32,
}

impl<'a> RenderContext for FocusIdRenderContext<'a> {
	fn focus_id(&self) -> i32 { self.focus_id }
	fn spot(&self) -> (i32, i32) { self.parent.spot() }
	fn yard_bounds(&self, yard_id: i32) -> Bounds { self.parent.yard_bounds(yard_id) }
	fn set_fill(&self, color: FillColor, z: i32) { self.parent.set_fill(color, z) }
	fn set_fill_grade(&self, fill_grade: FillGrade, z: i32) { self.parent.set_fill_grade(fill_grade, z) }
	fn set_glyph(&self, glyph: String, color: StrokeColor, z: i32) { self.parent.set_glyph(glyph, color, z) }
	fn set_dark(&self, z: i32) { self.parent.set_dark(z) }
}
