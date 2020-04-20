use std::ops::Deref;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

pub use multi_layout::*;
pub use yard::*;

use crate::yui::bounds::Bounds;
use crate::yui::palette::{FillColor, StrokeColor};

pub mod bounds;
pub mod layout;
pub mod glyph;
pub mod fill;
pub mod pad;
pub mod before;
pub mod palette;
pub mod pack;
pub mod place;
pub mod button;
pub mod confine;
pub mod empty;
pub mod tabbar;
pub(crate) mod yard;
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
	pub fn insert_char(&self, char: char, refresh: impl Fn() + Send + 'static) {
		match self.focus_type {
			FocusType::Submit => {}
			FocusType::Edit => {
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
			FocusType::Edit => {
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

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum FocusType {
	Submit,
	Edit,
}

pub fn render_submit(is_pressed: &Arc<RwLock<bool>>, ctx: &FocusActionContext) -> () {
	{
		*is_pressed.write().unwrap() = true;
	}
	ctx.refresh.deref()();
	thread::sleep(Duration::from_millis(100));
	{
		*is_pressed.write().unwrap() = false;
	}
	ctx.refresh.deref()();
}

pub trait RenderContext {
	fn focus_id(&self) -> i32;
	fn spot(&self) -> (i32, i32);
	fn yard_bounds(&self, yard_id: i32) -> Bounds;
	fn set_fill(&self, color: FillColor, z: i32);
	fn set_glyph(&self, glyph: char, color: StrokeColor, z: i32);
}

pub trait Padding {
	fn pad(self, size: i32) -> ArcYard;
	fn pad_cols(self, cols: i32) -> ArcYard;
}

pub trait Confine {
	fn confine_height(self, height: i32, cling: Cling) -> ArcYard;
	fn confine(self, width: i32, height: i32, cling: Cling) -> ArcYard;
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
	CenterMiddle,
	Left,
	LeftTop,
	Top,
}

impl Cling {
	fn x(&self) -> f32 {
		match self {
			Cling::Custom { x, .. } => { x.to_owned() }
			Cling::CenterMiddle => { 0.5 }
			Cling::Left => { 0.0 }
			Cling::LeftTop => { 0.0 }
			Cling::Top => { 0.5 }
		}
	}
	fn y(&self) -> f32 {
		match self {
			Cling::Custom { y, .. } => { y.to_owned() }
			Cling::CenterMiddle => { 0.5 }
			Cling::Left => { 0.5 }
			Cling::LeftTop => { 0.0 }
			Cling::Top => { 0.0 }
		}
	}
}

impl From<Cling> for (f32, f32) {
	fn from(cling: Cling) -> Self {
		(cling.x(), cling.y())
	}
}
