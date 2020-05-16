use std::ops::Deref;
use std::sync::{Arc, RwLock, RwLockReadGuard};
use std::thread;

use stringedit::StringEdit;

use crate::{Before, Focus, FocusAction, FocusMotion, FocusMotionFuture, FocusType, RenderContext};
use crate::yard::{ArcYard, Yard, YardOption};
use crate::yard;
use crate::yui::layout::LayoutContext;
use crate::yui::palette::{FillColor, StrokeColor};

pub fn textfield(id: i32, label: &str, edit: StringEdit, on_change: impl Fn(StringEdit) + 'static + Send + Sync) -> ArcYard {
	let yard = TextfieldYard {
		id,
		label_chars: label.chars().collect(),
		cow: Arc::new(CallOnWrite::new(edit, on_change)),
	};
	let arc_yard = Arc::new(yard) as ArcYard;
	arc_yard.before(yard::fill(FillColor::BackgroundWithFocus))
}

struct TextfieldYard {
	id: i32,
	label_chars: Vec<char>,
	cow: Arc<CallOnWrite<StringEdit>>,
}

struct CallOnWrite<T: Clone + Send + 'static> {
	value: RwLock<T>,
	change: Arc<dyn Fn(T) + 'static + Send + Sync>,
}


impl<T: Clone + Send + 'static> CallOnWrite<T> {
	fn set_value(&self, value: T) {
		(*self.value.write().unwrap()) = value.to_owned();
		let change = self.change.to_owned();
		thread::spawn(move || {
			(change)(value)
		});
	}
	fn value(&self) -> RwLockReadGuard<T> { self.value.read().unwrap() }
	fn new(value: T, on_change: impl Fn(T) + 'static + Send + Sync) -> Self {
		CallOnWrite {
			value: RwLock::new(value),
			change: Arc::new(on_change),
		}
	}
}

impl Yard for TextfieldYard {
	fn id(&self) -> i32 { self.id }

	fn update(&self, _option: YardOption) {}

	fn layout(&self, ctx: &mut LayoutContext) -> usize {
		let (edge_index, edge_bounds) = ctx.edge_bounds();
		let action_cow = self.cow.clone();
		let motion_cow = action_cow.clone();
		ctx.add_focus(Focus {
			yard_id: self.id,
			focus_type: FocusType::Edit(Arc::new(move |motion| {
				match motion {
					FocusMotion::Left => {
						let cursor_at_left = { motion_cow.value().cursor_index == 0 };
						if cursor_at_left {
							FocusMotionFuture::Default
						} else {
							let new_edit = { motion_cow.value().move_cursor_left() };
							motion_cow.set_value(new_edit);
							FocusMotionFuture::Skip
						}
					}
					FocusMotion::Right => {
						let cursor_at_right = {
							let guard = motion_cow.value();
							guard.cursor_index == guard.chars.len()
						};
						if cursor_at_right {
							FocusMotionFuture::Default
						} else {
							let new_edit = { motion_cow.value().move_cursor_right() };
							motion_cow.set_value(new_edit);
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
						let old_edit = {
							(*action_cow.value()).clone()
						};
						if !c.is_control() {
							action_cow.set_value(old_edit.insert_char(c));
							ctx.refresh.deref()();
						} else if c == '\x08' {
							action_cow.set_value(old_edit.delete_char_before_cursor());
							ctx.refresh.deref()();
						} else if c == '\x7f' {
							action_cow.set_value(old_edit.delete_char_at_cursor());
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
		let edit = self.cow.value();
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