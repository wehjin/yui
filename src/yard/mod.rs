use std::sync::Arc;

use crate::yui::layout::LayoutContext;
use crate::yui::palette::FillColor;
use crate::yui::RenderContext;

pub use self::button::*;
pub use self::empty::*;
pub use self::fade::*;
pub use self::fill::*;
pub use self::label::*;
pub use self::observable::*;
pub use self::story::*;
pub use self::textfield::*;

mod button;
mod story;
mod fade;
mod fill;
mod label;
mod textfield;
mod empty;
mod observable;

pub trait Yard {
	fn id(&self) -> i32;
	fn update(&self, option: YardOption);
	fn layout(&self, ctx: &mut LayoutContext) -> usize;
	fn render(&self, ctx: &dyn RenderContext);
}

pub enum YardOption {
	FillColor(FillColor)
}

pub type ArcYard = Arc<dyn Yard + Sync + Send>;

