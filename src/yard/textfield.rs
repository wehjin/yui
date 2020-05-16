use std::ops::Deref;
use std::sync::Arc;

use stringedit::StringEdit;

use crate::{Before, Focus, FocusAction, FocusMotion, FocusMotionFuture, FocusType, RenderContext};
use crate::yard::{ArcYard, Yard, YardOption};
use crate::yard;
use crate::yui::layout::LayoutContext;
use crate::yui::palette::{FillColor, StrokeColor};

pub fn textfield(id: i32, label: &str, edit: StringEdit, update: impl Fn(stringedit::Action) + 'static + Send + Sync) -> ArcYard {
	let yard = TextfieldYard {
		id,
		label_chars: label.chars().collect(),
		edit: Arc::new(edit),
		update: Arc::new(update),
	};
	let arc_yard = Arc::new(yard) as ArcYard;
	arc_yard.before(yard::fill(FillColor::BackgroundWithFocus))
}

struct TextfieldYard {
	id: i32,
	label_chars: Vec<char>,
	edit: Arc<StringEdit>,
	update: Arc<dyn Fn(stringedit::Action) + 'static + Send + Sync>,
}

impl Yard for TextfieldYard {
	fn id(&self) -> i32 { self.id }

	fn update(&self, _option: YardOption) {}

	fn layout(&self, ctx: &mut LayoutContext) -> usize {
		let (edge_index, edge_bounds) = ctx.edge_bounds();
		let focus_update = self.update.clone();
		let focus_edit = self.edit.clone();
		let action_update = self.update.clone();
		ctx.add_focus(Focus {
			yard_id: self.id,
			focus_type: FocusType::Edit(Arc::new(move |motion| {
				match motion {
					FocusMotion::Left => {
						let cursor_at_left = focus_edit.cursor_index == 0;
						if cursor_at_left {
							FocusMotionFuture::Default
						} else {
							(focus_update)(stringedit::Action::MoveCursorLeft);
							FocusMotionFuture::Skip
						}
					}
					FocusMotion::Right => {
						let cursor_at_right = focus_edit.cursor_index == focus_edit.chars.len();
						if cursor_at_right {
							FocusMotionFuture::Default
						} else {
							(focus_update)(stringedit::Action::MoveCursorRight);
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
						if !c.is_control() {
							(action_update)(stringedit::Action::InsertChar(c));
							ctx.refresh.deref()();
						} else if c == '\x08' {
							(action_update)(stringedit::Action::DeleteCharBeforeCursor);
							ctx.refresh.deref()();
						} else if c == '\x7f' {
							(action_update)(stringedit::Action::DeleteCharAtCursor);
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
		let edit = self.edit.clone();
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
					if edit.chars.len() > 0 {
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
			if ctx.focus_id() != self.id && edit.chars.len() == 0 {
				let char_index = col - body_bounds.left;
				if char_index >= 0 && (char_index as usize) < self.label_chars.len() {
					let char_index = char_index as usize;
					ctx.set_glyph(self.label_chars[char_index], StrokeColor::CommentOnBackground, body_bounds.z)
				}
			} else {
				let char_count = edit.chars.len();
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