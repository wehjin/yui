use std::ops::Deref;
use std::sync::{Arc, RwLock};

use crate::{ArcYard, DrawPad, Focus, FocusMotion, FocusMotionFuture, FocusType, MultiLayout};
use crate::yard::{Yard, YardOption};
use crate::yard::list::nexus::Nexus;
use crate::yui::bounds::Bounds;
use crate::layout::LayoutContext;
mod nexus;

pub fn list(id: i32, selected: usize, items: Vec<(u8, ArcYard)>) -> ArcYard {
	let mut item_tops = Vec::new();
	let mut item_heights = Vec::new();
	let mut sum_heights = 0i32;
	let mut yards = Vec::new();
	let mut min_item_height = 1000i32;
	for item in items {
		let (height, yard) = item;
		let height = height as i32;
		item_tops.push(sum_heights);
		sum_heights = sum_heights + height;
		item_heights.push(height);
		min_item_height = min_item_height.min(height);
		yards.push(yard);
	}
	let nexus = Arc::new(RwLock::new(Nexus::new(selected, &item_heights)));
	let sub_focus = Arc::new(RwLock::new(None));
	Arc::new(ListYard { id, item_tops, item_heights, min_item_height, sum_heights, yards, nexus, sub_focus })
}

struct ListYard {
	id: i32,
	item_tops: Vec<i32>,
	item_heights: Vec<i32>,
	min_item_height: i32,
	sum_heights: i32,
	yards: Vec<ArcYard>,
	nexus: Arc<RwLock<Nexus>>,
	sub_focus: Arc<RwLock<Option<Arc<Focus>>>>,
}

struct LayoutItem {
	index: usize,
	bounds: Bounds,
	yard: ArcYard,
}

impl Yard for ListYard {
	fn id(&self) -> i32 { self.id }
	fn update(&self, _option: YardOption) {}

	fn layout(&self, ctx: &mut LayoutContext) -> usize {
		let focus_index = self.nexus.read().expect("read nexus").item_index();
		let (_bounds_id, bounds) = ctx.edge_bounds();
		let mut focus = None;
		let final_bounds_id = {
			let mut multi_layout = MultiLayout::new(ctx);
			multi_layout.trap_foci(true);
			for layout_item in self.layout_items(&bounds) {
				multi_layout.layout(&layout_item.yard, &layout_item.bounds);
				if layout_item.index == focus_index {
					let sub_focus = multi_layout.trapped_focus().map(|it| Arc::new((*it).clone()));
					*self.sub_focus.write().expect("write sub_focus") = sub_focus.to_owned();
					focus = Some(self.create_focus(&bounds, sub_focus))
				}
			}
			multi_layout.finish()
		};
		if let Some(focus) = focus {
			ctx.add_focus(focus)
		}
		ctx.set_yard_bounds(self.id(), final_bounds_id);
		final_bounds_id
	}

	fn render(&self, bounds: &Bounds, focus_id: i32, _pad: &mut dyn DrawPad) -> Option<Vec<(ArcYard, Option<i32>)>> {
		let sub_focus_id = if let Some(sub_focus) = self.sub_focus.read().expect("read sub_focus").deref() {
			Some(sub_focus.yard_id)
		} else {
			None
		};
		let sub_focus_index = if focus_id == self.id {
			Some(self.nexus.read().expect("read nexus").item_index())
		} else {
			None
		};
		let more = self.layout_items(&bounds).iter().map(|layout_item| {
			let yard = layout_item.yard.clone();
			let focus_id = if Some(layout_item.index) == sub_focus_index && sub_focus_id.is_some() {
				let focus_id = sub_focus_id.expect("sub_focus_id");
				Some(focus_id)
			} else {
				None
			};
			(yard, focus_id)
		}).collect::<Vec<_>>();
		if more.is_empty() {
			None
		} else {
			Some(more)
		}
	}
}


impl ListYard {
	fn create_focus(&self, bounds: &Bounds, sub_focus: Option<Arc<Focus>>) -> Focus {
		let nexus = self.nexus.clone();
		let item_heights = self.item_heights.to_vec();
		let focus_motion = Arc::new(move |focus_motion| {
			let new_nexus = match focus_motion {
				FocusMotion::Left | FocusMotion::Right => None,
				FocusMotion::Up => nexus.read().expect("nexus read").up(&item_heights),
				FocusMotion::Down => nexus.read().expect("nexus read").down(&item_heights),
			};
			match new_nexus {
				None => FocusMotionFuture::Default,
				Some(new_nexus) => {
					*nexus.write().expect("nexus write") = new_nexus;
					FocusMotionFuture::Skip
				}
			}
		});
		let focus_type = match &sub_focus {
			None => FocusType::CompositeSubmit(focus_motion),
			Some(focus) => {
				match &focus.focus_type {
					FocusType::Submit => FocusType::CompositeSubmit(focus_motion),
					FocusType::Edit(_) => FocusType::Edit(focus_motion),
					FocusType::CompositeSubmit(_) => FocusType::CompositeSubmit(focus_motion),
				}
			}
		};
		let focus = Focus {
			yard_id: self.id,
			focus_type,
			bounds: bounds.to_owned(),
			priority: 0,
			action_block: Arc::new(move |ctx| {
				if let Some(sub_focus) = &sub_focus {
					(*sub_focus.action_block)(ctx);
				}
			}),
		};
		focus
	}

	fn layout_items(&self, bounds: &Bounds) -> Vec<LayoutItem> {
		let nexus = self.nexus.read().expect("nexus read");
		let pivot_row = nexus.pivot_row(
			bounds.height(),
			bounds.top,
			self.sum_heights,
			self.min_item_height,
			&self.item_tops,
		);
		let pivot_pos = nexus.pivot_pos();
		let mut layout_items = Vec::new();
		let mut next_index = Some(0);
		while next_index.is_some() {
			let index = next_index.expect("next_index");
			next_index = if index >= self.item_heights.len() {
				None
			} else {
				let item_bounds = nexus.item_bounds(index, bounds, pivot_row, pivot_pos, &self.item_tops, &self.item_heights);
				let (next, keep) = if item_bounds.bottom < bounds.top {
					// Full underflow
					(Some(index + 1), false)
				} else if item_bounds.top < bounds.top {
					// Partial underflow and possibly overflow
					if item_bounds.bottom > bounds.bottom {
						// Full overlap
						(None, true)
					} else {
						// Partial underflow
						(Some(index + 1), true)
					}
				} else if item_bounds.bottom <= bounds.bottom {
					// In bounds
					(Some(index + 1), true)
				} else if item_bounds.top < bounds.bottom {
					// Partial overflow
					(None, true)
				} else {
					// Full overflow
					(None, false)
				};
				if keep {
					let layout_item = LayoutItem {
						index,
						bounds: item_bounds,
						yard: self.yards[index].clone(),
					};
					layout_items.push(layout_item);
				}
				next
			}
		}
		layout_items
	}
}
