use std::sync::Arc;

pub use fill::*;
pub use label::*;
pub use textfield::*;

use crate::yui::layout::LayoutContext;
use crate::yui::palette::FillColor;
use crate::yui::RenderContext;

mod label;
mod textfield;
mod fill;


pub type ArcYard = Arc<dyn Yard + Sync + Send>;

pub trait Yard {
	fn id(&self) -> i32;
	fn update(&self, option: YardOption);
	fn layout(&self, ctx: &mut LayoutContext) -> usize;
	fn render(&self, ctx: &dyn RenderContext);
}

pub enum YardOption {
	FillColor(FillColor)
}

