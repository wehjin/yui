use std::error::Error;
use std::sync::Arc;
use std::sync::mpsc::Receiver;

use crate::yui::layout::LayoutContext;
use crate::yui::palette::FillColor;
use crate::yui::RenderContext;

pub use self::fade::*;
pub use self::fill::*;
pub use self::label::*;
pub use self::story::*;
pub use self::textfield::*;

mod story;
mod fade;
mod fill;
mod label;
mod textfield;

pub trait YardObservableSource {
	fn yards(&self) -> Box<dyn YardObservable>;
}

pub trait YardObservable {
	fn subscribe(&self) -> Result<Receiver<ArcYard>, Box<dyn Error>>;
}

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

