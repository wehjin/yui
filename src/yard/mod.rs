use std::sync::Arc;

use crate::yui::layout::LayoutContext;
use crate::yui::palette::FillColor;
use crate::yui::RenderContext;

pub use self::button::*;
pub use self::empty::*;
pub use self::fade::*;
pub use self::fill::*;
pub use self::glyph::*;
pub use self::label::*;
pub use self::observable::*;
pub use self::quad_label::*;
pub use self::story::*;
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
mod story;
mod textfield;
mod title;
mod trellis;

mod quad_label;

pub trait Yard {
	fn id(&self) -> i32;
	fn update(&self, option: YardOption);
	fn layout(&self, ctx: &mut LayoutContext) -> usize;
	fn render(&self, ctx: &dyn RenderContext);
}

pub type ArcYard = Arc<dyn Yard + Sync + Send>;

pub enum YardOption {
	FillColor(FillColor)
}

pub type ArcEvent<T> = Arc<dyn Fn(T) + Send + Sync + 'static>;
pub type ArcTouch = Arc<dyn Fn() + Send + Sync + 'static>;

pub fn ignore_touch() -> ArcTouch { Arc::new(|| ()) }
