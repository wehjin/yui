use crate::{ArcYard, Cling, Confine, Pack, yard};
use crate::yui::palette::StrokeColor;

pub fn title<T: AsRef<str>>(text: T, color: StrokeColor, cling: Cling) -> ArcYard {
	let text = text.as_ref();
	let length = text.chars().count();
	let title = yard::label(text, color, cling);
	let line = yard::glyph(color, || '=').confine(length as i32, 1, cling);
	title.pack_bottom(1, line).confine_height(2, Cling::Top)
}