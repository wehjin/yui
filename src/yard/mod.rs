use std::sync::Arc;

use crate::palette::FillColor;
use crate::yui::layout::LayoutContext;
use crate::yui::RenderContext;

pub use self::button::*;
pub use self::empty::*;
pub use self::fade::*;
pub use self::fill::*;
pub use self::glyph::*;
pub use self::label::*;
pub use self::list::*;
pub use self::observable::*;
pub use self::pressable::*;
pub use self::quad_label::*;
pub use self::story::*;
pub use self::tabbar::*;
pub use self::textfield::*;
pub use self::title::*;
pub use self::trellis::*;

mod button;
mod empty;
mod fade;
mod fill;
mod glyph;
mod label;
mod observable;
mod pressable;
mod quad_label;
mod list;
mod story;
mod tabbar;
mod textfield;
mod title;
mod trellis;

pub trait Yard {
	fn render(&self, ctx: &dyn RenderContext);
	fn layout(&self, ctx: &mut LayoutContext) -> usize;
	fn update(&self, _option: YardOption) {}
	fn id(&self) -> i32;
}

pub type ArcYard = Arc<dyn Yard + Sync + Send>;

pub enum YardOption {
	FillColor(FillColor)
}

pub type ArcEvent<T> = Arc<dyn Fn(T) + Send + Sync + 'static>;
pub type ArcTouch = Arc<dyn Fn() + Send + Sync + 'static>;

pub fn ignore_touch() -> ArcTouch { Arc::new(|| ()) }
