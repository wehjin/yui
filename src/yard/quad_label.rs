use crate::{ArcYard, Cling, Pack, yard};
use crate::palette::{body_and_comment_for_fill, FillColor};

pub fn quad_label(title: &str, subtitle: &str, value: &str, subvalue: &str, value_cols: usize, fill_color: FillColor) -> ArcYard {
	let (color, subcolor) = body_and_comment_for_fill(fill_color);
	let title = yard::label(title, color, Cling::LeftTop);
	let subtitle = yard::label(subtitle, subcolor, Cling::LeftBottom);
	let value = yard::label(value, color, Cling::RightTop);
	let subvalue = yard::label(subvalue, subcolor, Cling::RightBottom);
	let left = title.pack_bottom(1, subtitle);
	let right = value.pack_bottom(1, subvalue);
	left.pack_right(value_cols as i32, right)
}
