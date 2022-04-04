use std::sync::{Arc, RwLock};

use crate::{Bounds, DrawPad, Link, SenderLink, SyncLink};
use crate::palette::FillGrade;
use crate::yard::{ArcTouch, ArcYard, Yard};
use crate::yui::{Focus, FocusType, render_submit};
use crate::layout::LayoutContext;
pub trait Pressable {
	fn pressable(self, on_press: SenderLink<i32>) -> ArcYard;
}

impl Pressable for ArcYard {
	fn pressable(self, on_press: SenderLink<i32>) -> ArcYard { pressable(self, on_press) }
}

pub fn pressable(yard: ArcYard, on_press: SenderLink<i32>) -> ArcYard {
	let id = rand::random();
	let is_pressed = Arc::new(RwLock::new(false));
	let on_press = Arc::new({
		let sync_press: SyncLink<i32> = on_press.into();
		move || sync_press.send(id)
	});
	Arc::new(PressYard { id, yard, is_pressed, on_press })
}

struct PressYard {
	id: i32,
	yard: ArcYard,
	is_pressed: Arc<RwLock<bool>>,
	on_press: ArcTouch,
}

impl Yard for PressYard {
	fn id(&self) -> i32 { self.id }

	fn layout(&self, ctx: &mut LayoutContext) -> usize {
		let (_edge_index, edge_bounds) = ctx.edge_bounds();
		let core_bounds = edge_bounds.with_z_far_z(edge_bounds.z - 1, edge_bounds.z);
		let core_index_start = ctx.push_bounds(&core_bounds);
		let core_index_finish = self.yard.layout(&mut ctx.with_index(core_index_start).trap_foci());
		{
			let is_pressed = self.is_pressed.clone();
			let on_press = self.on_press.to_owned();
			ctx.add_focus(Focus {
				yard_id: self.id(),
				focus_type: FocusType::Submit,
				bounds: edge_bounds,
				priority: 0,
				action_block: Arc::new(move |ctx| render_submit(&is_pressed, ctx, &on_press)),
			});
		}
		ctx.set_yard_bounds(self.id, core_index_finish)
	}

	fn render(&self, bounds: &Bounds, focus_id: i32, pad: &mut dyn DrawPad) -> Option<Vec<(ArcYard, Option<i32>)>> {
		if focus_id == self.id {
			let grade = if self.is_pressed() { FillGrade::Press } else { FillGrade::Focus };
			pad.grade(bounds, grade);
		}
		Some(vec![(self.yard.clone(), None)])
	}
}

impl PressYard {
	fn is_pressed(&self) -> bool {
		*self.is_pressed.read().expect("read is_pressed")
	}
}
