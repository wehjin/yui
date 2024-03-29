use std::sync::Arc;

pub use basic::empty::*;
pub use basic::fade::*;
pub use basic::fill::*;
pub use basic::glyph::*;
pub use basic::label::*;
pub use basic::story::*;
pub use scalar::button::*;

use crate::core::bounds::Bounds;
use crate::DrawPad;
use crate::layout::LayoutContext;
use crate::palette::{FillColor, FillGrade};

pub use self::grade::*;
pub use self::list::*;
pub use self::mux::*;
pub use self::observable::*;
pub use scalar::pressable::*;
pub use self::quad_label::*;
pub use self::tabbar::*;
pub use self::table::*;
pub use self::textfield::*;
pub use self::title::*;
pub use self::trellis::*;

mod grade;
mod mux;
mod observable;
mod quad_label;
mod list;
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
