use std::sync::Arc;

pub use basic::fade::*;
pub use basic::glyph::*;
pub use basic::label::*;
pub use basic::story::*;
pub use scalar::button::*;
pub use scalar::button2::*;

use crate::bounds::Bounds;
use crate::DrawPad;
use crate::layout::LayoutContext;
use crate::palette::{FillColor, FillGrade};

pub use self::empty::*;
pub use self::fill::*;
pub use self::grade::*;
pub use self::list::*;
pub use self::mux::*;
pub use self::observable::*;
pub use self::pressable::*;
pub use self::publisher::*;
pub use self::quad_label::*;
pub use self::tabbar::*;
pub use self::table::*;
pub use self::textfield::*;
pub use self::title::*;
pub use self::trellis::*;

mod empty;
mod fill;
mod grade;
mod mux;
mod observable;
mod pressable;
mod quad_label;
mod list;
mod publisher;
mod tabbar;
mod table;
mod textfield;
mod title;
mod trellis;
mod basic;
mod scalar;

pub trait Yard {
	fn id(&self) -> i32;
	fn type_desc(&self) -> &'static str { "" }
	fn desc(&self) -> String { format!("{}Yard {{ id:{} }}", self.type_desc(), self.id()) }

	// TODO: Delete this
	fn update(&self, _option: YardOption) {}

	fn layout(&self, ctx: &mut LayoutContext) -> usize;
	fn render(&self, _bounds: &Bounds, _focus_id: i32, _pad: &mut dyn DrawPad) -> Option<Vec<(ArcYard, Option<i32>)>>;
}


pub type ArcYard = Arc<dyn Yard + Sync + Send>;

pub enum YardOption {
	FillColor(FillColor, FillGrade)
}

pub type ArcEvent<T> = Arc<dyn Fn(T) + Send + Sync + 'static>;
pub type ArcTouch = Arc<dyn Fn() + Send + Sync + 'static>;

pub fn ignore_touch() -> ArcTouch { Arc::new(|| ()) }
