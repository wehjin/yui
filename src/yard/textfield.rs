use std::ops::Deref;
use std::sync::Arc;

use stringedit::StringEdit;

use crate::{Before, Bounds, DrawPad, Focus, FocusAction, FocusMotion, FocusMotionFuture, FocusType, Link, SenderLink, StrokeColor, SyncLink};
use crate::layout::LayoutContext;
use crate::palette::{FillColor, FillGrade};
use crate::yard::{ArcYard, Yard, YardOption};
use crate::yard;

pub fn textfield(id: i32, label: &str, edit: StringEdit, update: SenderLink<stringedit::Action>) -> ArcYard {
	let yard = TextfieldYard {
		id,
		label_chars: label.chars().collect(),
		edit: Arc::new(edit),
		update: update.into(),
	};
	let arc_yard = Arc::new(yard) as ArcYard;
	arc_yard.before(yard::fill(FillColor::Background, FillGrade::Select))
}

struct TextfieldYard {
	id: i32,
	label_chars: Vec<char>,
	edit: Arc<StringEdit>,
	update: SyncLink<stringedit::Action>,
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
							focus_update.send(stringedit::Action::MoveCursorLeft);
							FocusMotionFuture::Skip
						}
					}
					FocusMotion::Right => {
						let cursor_at_right = focus_edit.cursor_index == focus_edit.chars.len();
						if cursor_at_right {
							FocusMotionFuture::Default
						} else {
							focus_update.send(stringedit::Action::MoveCursorRight);
							FocusMotionFuture::Skip
						}
					}
					FocusMotion::Up => FocusMotionFuture::Default,
					FocusMotion::Down => FocusMotionFuture::Default,
				}
			})),
			bounds: edge_bounds.clone(),
			priority: 0,
			action_block: Arc::new(move |ctx| {
				match ctx.action {
					FocusAction::Go => {}
					FocusAction::Change(c) => {
						if !c.is_control() {
							action_update.send(stringedit::Action::InsertChar(c));
							ctx.refresh.deref()();
						} else if c == '\x08' {
							action_update.send(stringedit::Action::DeleteCharBeforeCursor);
							ctx.refresh.deref()();
						} else if c == '\x7f' {
							action_update.send(stringedit::Action::DeleteCharAtCursor);
							ctx.refresh.deref()();
						}
					}
				}
			}),
		});
		ctx.set_yard_bounds(self.id, edge_index);
		edge_index
	}


	fn render(&self, edge_bounds: &Bounds, focus_id: i32, pad: &mut dyn DrawPad) -> Option<Vec<(ArcYard, Option<i32>)>> {
		let (head_bounds, lower_bounds) = edge_bounds.split_from_top(1);
		self.render_head(&head_bounds, focus_id, pad);

		let (body_bounds, foot_bounds) = lower_bounds.split_from_bottom(1);
		self.render_foot(&foot_bounds, focus_id, pad);
		self.render_body(&body_bounds, focus_id, pad);
		None
	}
}

impl TextfieldYard {
	fn render_head(&self, bounds: &Bounds, focus_id: i32, pad: &mut dyn DrawPad) {
		let bounds = bounds.pad(1, 1, 0, 0);
		if !self.label_chars.is_empty() {
			let glyph: String = self.label_chars.iter().collect();
			if focus_id == self.id {
				pad.glyph(&bounds, &glyph, StrokeColor::EnabledOnBackground);
			} else {
				if self.edit.chars.is_empty() {
					// Do nothing. The label will appear in the body.
				} else {
					pad.glyph(&bounds, &glyph, StrokeColor::CommentOnBackground);
				}
			}
		}
	}
	fn render_body(&self, bounds: &Bounds, focus_id: i32, pad: &mut dyn DrawPad) {
		let bounds = bounds.pad(1, 1, 0, 0);
		let edit = self.edit.clone();
		if focus_id != self.id && edit.chars.len() == 0 {
			if !self.label_chars.is_empty() {
				let label: String = self.label_chars.iter().collect();
				pad.glyph(&bounds, &label, StrokeColor::CommentOnBackground);
			}
		} else {
			let cursor_from_left = {
				let full_length = edit.chars.len();
				let visible_length = bounds.width() as usize;
				if full_length < visible_length {
					edit.cursor_index as i32
				} else {
					let cursor_fraction = edit.cursor_index as f32 / full_length as f32;
					let range = (visible_length - 1) as f32;
					(cursor_fraction * range).ceil() as i32
				}
			};
			{
				let pre_chars: Vec<char> = edit.chars[0..edit.cursor_index].iter().cloned().collect();
				if !pre_chars.is_empty() {
					let pre_start = (pre_chars.len() as i32 - cursor_from_left).max(0) as usize;
					let pre_visible_chars: String = pre_chars[pre_start..].iter().cloned().map(|c| if !c.is_control() { c } else { ' ' }).collect();
					pad.glyph(&bounds, &pre_visible_chars, StrokeColor::BodyOnBackground);
				}
			}
			{
				let cursor_bounds = bounds.pad(cursor_from_left, bounds.width() - (cursor_from_left + 1), 0, 0);
				if focus_id == self.id {
					pad.fill(&cursor_bounds, FillColor::Background);
					pad.grade(&cursor_bounds, FillGrade::Focus);
				}
				if edit.cursor_index < edit.chars.len() {
					let char = edit.chars[edit.cursor_index];
					if !char.is_control() {
						pad.glyph(&cursor_bounds, &char.to_string(), StrokeColor::BodyOnBackground);
					}
				};
			}
			{
				let post_start = edit.cursor_index + 1;
				if post_start < edit.chars.len() {
					let post_chars: String = edit.chars[post_start..].iter().cloned().map(|c| if !c.is_control() { c } else { ' ' }).collect();
					let post_bounds = bounds.pad(cursor_from_left + 1, 0, 0, 0);
					pad.glyph(&post_bounds, &post_chars, StrokeColor::BodyOnBackground);
				}
			}
		}
	}
	fn render_foot(&self, bounds: &Bounds, focus_id: i32, pad: &mut dyn DrawPad) {
		let color = if focus_id == self.id {
			StrokeColor::EnabledOnBackground
		} else {
			StrokeColor::CommentOnBackground
		};
		let underline: String = (0..bounds.width()).map(|_| '_').collect();
		pad.glyph(bounds, &underline, color);
	}
}

#[cfg(test)]
mod tests {
	use stringedit::{StringEdit, Validity};

	use crate::{layout, render, SenderLink, yard};
	use crate::FillColor::Background;
	use crate::FillGrade::{Focus, Select};
	use crate::StrokeColor::{BodyOnBackground, EnabledOnBackground};
	use crate::yui::layout::ActiveFocus;

	#[test]
	fn layout_render() {
		let edit = StringEdit::new("words", 5, Validity::NotEmpty);
		let yard = yard::textfield(300, "Label", edit, SenderLink::ignore());
		let (max_x, max_y) = (4, 3);
		let layout = layout::run(max_y, max_x, &yard, &ActiveFocus::default());
		let spot_table = render::run(&yard, layout.max_x, layout.max_y, layout.bounds_hold.clone(), layout.active_focus.focus_id());
		let fronts = spot_table.to_fronts();


		let computed = fronts.iter().flatten()
			.map(|front| (front.fill_color.clone(), front.stroke.clone(), front.fill_grade))
			.collect::<Vec<_>>();
		assert_eq!(computed[0..4], vec![
			(Background, None, Select),
			(Background, Some(('L'.into(), EnabledOnBackground)), Select),
			(Background, Some(('a'.into(), EnabledOnBackground)), Select),
			(Background, None, Select),
		]);
		assert_eq!(computed[4..8], vec![
			(Background, None, Select),
			(Background, Some(('s'.into(), BodyOnBackground)), Select),
			(Background, None, Focus),
			(Background, None, Select),
		]);
		assert_eq!(computed[8..12], vec![
			(Background, Some(('_'.into(), EnabledOnBackground)), Select),
			(Background, Some(('_'.into(), EnabledOnBackground)), Select),
			(Background, Some(('_'.into(), EnabledOnBackground)), Select),
			(Background, Some(('_'.into(), EnabledOnBackground)), Select),
		]);
	}
}