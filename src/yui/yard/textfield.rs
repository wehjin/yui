use std::sync::Arc;

use stringedit::StringEdit;

use crate::yui::{ArcYard, Before, Focus, FocusType, RenderContext, Yard, YardOption};
use crate::yui::fill::fill_yard;
use crate::yui::layout::LayoutContext;
use crate::yui::palette::{FillColor, StrokeColor};

pub fn textfield(label: &str) -> ArcYard {
	let yard = TextfieldYard {
		id: rand::random(),
		label_chars: label.chars().collect(),
		edit: StringEdit::empty(),
	};
	let arc_yard = Arc::new(yard) as ArcYard;
	arc_yard.before(fill_yard(FillColor::BackgroundWithFocus))
}

struct TextfieldYard {
	id: i32,
	label_chars: Vec<char>,
	edit: StringEdit,
}

impl Yard for TextfieldYard {
	fn id(&self) -> i32 { self.id }

	fn update(&self, _option: YardOption) {}

	fn layout(&self, ctx: &mut LayoutContext) -> usize {
		let (edge_index, edge_bounds) = ctx.edge_bounds();
		ctx.add_focus(Focus {
			yard_id: self.id,
			focus_type: FocusType::Edit,
			bounds: edge_bounds.clone(),
			action_block: Arc::new(|_ctx| {}),
		});
		ctx.set_yard_bounds(self.id, edge_index);
		edge_index
	}

	fn render(&self, ctx: &dyn RenderContext) {
		let (row, col) = ctx.spot();
		let edge_bounds = ctx.yard_bounds(self.id);
		let (head_bounds, lower_bounds) = edge_bounds.split_from_top(1);
		let head_bounds = head_bounds.pad(1, 1, 0, 0);
		if head_bounds.intersects(row, col) {
			let char_index = col - head_bounds.left;
			if char_index >= 0 && (char_index as usize) < self.label_chars.len() {
				let char_index = char_index as usize;
				if ctx.focus_id() == self.id {
					ctx.set_glyph(self.label_chars[char_index], StrokeColor::EnabledOnBackground, head_bounds.z)
				} else {
					if self.edit.char_count() > 0 {
						ctx.set_glyph(self.label_chars[char_index], StrokeColor::CommentOnBackground, head_bounds.z)
					}
				}
			}
		}
		let (body_bounds, foot_bounds) = lower_bounds.split_from_bottom(1);
		if foot_bounds.intersects(row, col) {
			let color = if ctx.focus_id() == self.id {
				StrokeColor::EnabledOnBackground
			} else {
				StrokeColor::CommentOnBackground
			};
			ctx.set_glyph('_', color, foot_bounds.z)
		}
		let body_bounds = body_bounds.pad(1, 1, 0, 0);
		if body_bounds.intersects(row, col) {
			if ctx.focus_id() != self.id && self.edit.char_count() == 0 {
				let char_index = col - body_bounds.left;
				if char_index >= 0 && (char_index as usize) < self.label_chars.len() {
					let char_index = char_index as usize;
					ctx.set_glyph(self.label_chars[char_index], StrokeColor::CommentOnBackground, body_bounds.z)
				}
			} else {
				let raw_char_count = self.edit.char_count();
				let target_width = (body_bounds.width() - 1).max(0);
				let char_count = (raw_char_count as i32 + 1).max(target_width);
				let cursor_fraction = (self.edit.cursor_index as f32) / char_count as f32;
				let cursor_distance_from_left = (cursor_fraction * (target_width as f32)) as i32;
				let cursor_x = body_bounds.left + cursor_distance_from_left;
				let col_dist_from_cursor = col - cursor_x;
				let char_index = (self.edit.cursor_index as i32) + col_dist_from_cursor;
				if char_index >= 0 && char_index <= raw_char_count as i32 {
					if let Some(spot) = self.edit.read_spot(char_index as usize) {
						if spot.is_cursor && ctx.focus_id() == self.id {
							ctx.set_fill(FillColor::BackgroundWithPress, body_bounds.z);
						}
						if spot.char != '\n' {
							ctx.set_glyph(spot.char, StrokeColor::BodyOnBackground, body_bounds.z);
						}
					}
				}
			}
		}
	}
}