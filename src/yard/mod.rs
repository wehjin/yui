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

mod quad_label {
	use crate::{ArcYard, Cling, Pack, yard};
	use crate::yui::palette::{body_and_comment_for_fill, FillColor};

	pub fn quad_label(title: &str, subtitle: &str, value: &str, subvalue: &str, value_rows: usize, fill_color: FillColor) -> ArcYard {
		let (color, subcolor) = body_and_comment_for_fill(fill_color);
		let title = yard::label(title, color, Cling::LeftTop);
		let subtitle = yard::label(subtitle, subcolor, Cling::LeftBottom);
		let value = yard::label(value, color, Cling::RightTop);
		let subvalue = yard::label(subvalue, subcolor, Cling::RightBottom);
		let left = title.pack_bottom(1, subtitle);
		let right = value.pack_bottom(1, subvalue);
		left.pack_right(value_rows as i32, right)
	}
}

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
