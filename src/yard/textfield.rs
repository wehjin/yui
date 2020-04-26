use std::ops::Deref;
use std::sync::{Arc, RwLock};

use stringedit::StringEdit;

use crate::{Before, Focus, FocusAction, FocusMotion, FocusMotionFuture, FocusType, RenderContext};
use crate::yard::{ArcYard, Yard, YardOption};
use crate::yard;
use crate::yui::layout::LayoutContext;
use crate::yui::palette::{FillColor, StrokeColor};

pub fn textfield(label: &str) -> ArcYard {
	let yard = TextfieldYard {
		id: rand::random(),
		label_chars: label.chars().collect(),
		edit: Arc::new(RwLock::new(StringEdit::empty())),
	};
	let arc_yard = Arc::new(yard) as ArcYard;
	arc_yard.before(yard::fill(FillColor::BackgroundWithFocus))
}

struct TextfieldYard {
	id: i32,
	label_chars: Vec<char>,
	edit: Arc<RwLock<StringEdit>>,
}

impl Yard for TextfieldYard {
	fn id(&self) -> i32 { self.id }

	fn update(&self, _option: YardOption) {}

	fn layout(&self, ctx: &mut LayoutContext) -> usize {
		let (edge_index, edge_bounds) = ctx.edge_bounds();
		let edit = self.edit.clone();
		let motion_edit = edit.clone();
		ctx.add_focus(Focus {
			yard_id: self.id,
			focus_type: FocusType::Edit(Arc::new(move |motion| {
				match motion {
					FocusMotion::Left => {
						let cursor_at_left = { motion_edit.read().unwrap().cursor_index == 0 };
						if cursor_at_left {
							FocusMotionFuture::Default
						} else {
							let new_edit = { motion_edit.read().unwrap().move_cursor_left() };
							*motion_edit.write().unwrap() = new_edit;
							FocusMotionFuture::Skip
						}
					}
					FocusMotion::Right => {
						let cursor_at_right = {
							let guard = motion_edit.read().unwrap();
							guard.cursor_index == guard.char_count()
						};
						if cursor_at_right {
							FocusMotionFuture::Default
						} else {
							let new_edit = { motion_edit.read().unwrap().move_cursor_right() };
							*motion_edit.write().unwrap() = new_edit;
							FocusMotionFuture::Skip
						}
					}
					FocusMotion::Up => FocusMotionFuture::Default,
					FocusMotion::Down => FocusMotionFuture::Default,
				}
			})),
			bounds: edge_bounds.clone(),
			action_block: Arc::new(move |ctx| {
				match ctx.action {
					FocusAction::Go => {}
					FocusAction::Change(c) => {
						let old_edit = { (*edit.read().unwrap()).clone() };
						if !c.is_control() {
							*edit.write().unwrap() = old_edit.insert_char(c);
							ctx.refresh.deref()();
						} else if c == '\x08' {
							*edit.write().unwrap() = old_edit.delete_char_before_cursor();
							ctx.refresh.deref()();
						} else if c == '\x7f' {
							*edit.write().unwrap() = old_edit.delete_char_at_cursor();
							ctx.refresh.deref()();
						}
					}
				}
			}),
		});
		ctx.set_yard_bounds(self.id, edge_index);
		edge_index
	}

	fn render(&self, ctx: &dyn RenderContext) {
		let (row, col) = ctx.spot();
		let edit = self.edit.read().unwrap();
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
					if edit.char_count() > 0 {
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
			if ctx.focus_id() != self.id && edit.char_count() == 0 {
				let char_index = col - body_bounds.left;
				if char_index >= 0 && (char_index as usize) < self.label_chars.len() {
					let char_index = char_index as usize;
					ctx.set_glyph(self.label_chars[char_index], StrokeColor::CommentOnBackground, body_bounds.z)
				}
			} else {
				let char_count = edit.char_count();
				let visible_chars = body_bounds.width() as usize;
				let cursor_from_left = if char_count < visible_chars {
					edit.cursor_index as i32
				} else {
					let cursor_fraction = edit.cursor_index as f32 / char_count as f32;
					let range = (visible_chars - 1) as f32;
					(cursor_fraction * range).ceil() as i32
				};
				let cursor_x = body_bounds.left + cursor_from_left;
				let cursor_to_col = col - cursor_x;
				let char_index = (edit.cursor_index as i32) + cursor_to_col;
				if char_index >= 0 && char_index <= char_count as i32 {
					if let Some(spot) = edit.read_spot(char_index as usize) {
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