use std::ops::Deref;
use std::sync::{Arc, RwLock};

use crate::{ArcYard, DrawPad, Focus, FocusMotion, FocusMotionFuture, FocusType, Link, MultiLayout, SyncLink};
use crate::layout::LayoutContext;
use crate::yard::{Yard};
use crate::yard::list::nexus::Nexus;
use crate::yard::model::{ScrollAction, ScrollModel};
use crate::core::bounds::Bounds;

mod nexus;

pub mod model {
	use crate::yard::list::nexus::Nexus;

	#[derive(Debug, Copy, Clone)]
	pub enum ScrollAction { Up, Down }

	#[derive(Debug, Clone)]
	pub struct ScrollModel {
		pub id: i32,
		pub item_heights: Vec<i32>,
		pub item_tops: Vec<i32>,
		pub min_item_height: i32,
		pub sum_heights: i32,
		pub nexus: Nexus,
		pub selected_index: usize,
	}

	impl ScrollModel {
		pub fn new_count_height(id: i32, count: usize, height: u8, selected_index: usize) -> Self {
			let heights = (0..count).map(|_| height).collect::<Vec<_>>();
			Self::new(id, heights, selected_index)
		}
		pub fn new(id: i32, item_heights: Vec<u8>, selected_index: usize) -> Self {
			let item_heights = item_heights.into_iter().map(|it| it as i32).collect::<Vec<_>>();
			let mut item_tops = Vec::new();
			let mut min_item_height = i32::MAX;
			let mut sum_heights = 0i32;
			for item_height in &item_heights {
				let item_height = *item_height;
				item_tops.push(sum_heights);
				sum_heights += item_height;
				min_item_height = min_item_height.min(item_height);
			}
			let nexus = Nexus::new(0, &item_heights);
			ScrollModel { id, item_heights, item_tops, min_item_height, sum_heights, nexus, selected_index }
		}
		pub fn item_count(&self) -> usize { self.item_heights.len() }
		pub fn selected_index(&self) -> usize { self.selected_index }
		pub fn update(&self, action: ScrollAction) -> Option<Self> {
			match action {
				ScrollAction::Up => self.nexus.up(&self.item_heights).map(|it| self.with_nexus(it)),
				ScrollAction::Down => self.nexus.down(&self.item_heights).map(|it| self.with_nexus(it)),
			}
		}
		pub fn with_nexus(&self, nexus: Nexus) -> Self {
			let mut art = self.clone();
			art.nexus = nexus;
			art
		}
		pub fn with_selected_index(&self, index: usize) -> Self {
			let mut new = self.clone();
			new.selected_index = index;
			new
		}
	}
}

pub fn list(yards: Vec<ArcYard>, scroll: ScrollModel, scroll_link: SyncLink<ScrollAction>) -> ArcYard {
	assert_eq!(scroll.item_count(), yards.len());
	let sub_focus = Arc::new(RwLock::new(None));
	Arc::new(ListYard { scroll, yards, sub_focus, scroll_link })
}

struct ListYard {
	scroll: ScrollModel,
	yards: Vec<ArcYard>,
	scroll_link: SyncLink<ScrollAction>,
	sub_focus: Arc<RwLock<Option<Arc<Focus>>>>,
}

struct LayoutItem {
	index: usize,
	bounds: Bounds,
	yard: ArcYard,
}

impl Yard for ListYard {
	fn id(&self) -> i32 { self.scroll.id }

	fn layout(&self, ctx: &mut LayoutContext) -> usize {
		let (_bounds_id, bounds) = ctx.edge_bounds();
		let mut focus = None;
		let final_bounds_id = {
			let mut multi_layout = MultiLayout::new(ctx);
			multi_layout.trap_foci(true);
			let focus_index = self.scroll.nexus.item_index();
			for layout_item in self.layout_items(&bounds) {
				multi_layout.layout(&layout_item.yard, &layout_item.bounds);
				if layout_item.index == focus_index {
					let sub_focus = multi_layout.trapped_focus().map(|it| Arc::new((*it).clone()));
					*self.sub_focus.write().expect("write sub_focus") = sub_focus.to_owned();
					focus = Some(self.create_focus(&bounds, sub_focus, &self.scroll.nexus, self.scroll_link.clone()))
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
		let sub_focus_index = if focus_id == self.scroll.id {
			Some(self.scroll.nexus.item_index())
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
	fn create_focus(&self, bounds: &Bounds, sub_focus: Option<Arc<Focus>>, nexus: &Nexus, list_link: SyncLink<ScrollAction>) -> Focus {
		let can_up = nexus.can_up();
		let can_down = nexus.can_down();
		let focus_motion = Arc::new(move |focus_motion| {
			match focus_motion {
				FocusMotion::Left | FocusMotion::Right => {
					FocusMotionFuture::Default
				}
				FocusMotion::Up => {
					if can_up {
						list_link.send(ScrollAction::Up);
						FocusMotionFuture::Skip
					} else {
						FocusMotionFuture::Default
					}
				}
				FocusMotion::Down => {
					if can_down {
						list_link.send(ScrollAction::Down);
						FocusMotionFuture::Skip
					} else {
						FocusMotionFuture::Default
					}
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
			yard_id: self.scroll.id,
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
		let nexus = &self.scroll.nexus;
		let pivot_row = nexus.pivot_row(
			bounds.height(),
			bounds.top,
			self.scroll.sum_heights,
			self.scroll.min_item_height,
			&self.scroll.item_tops,
		);
		let pivot_pos = nexus.pivot_pos();
		let mut layout_items = Vec::new();
		let mut next_index = Some(0);
		while next_index.is_some() {
			let index = next_index.expect("next_index");
			next_index = if index >= self.scroll.item_heights.len() {
				None
			} else {
				let item_bounds = nexus.item_bounds(index, bounds, pivot_row, pivot_pos, &self.scroll.item_tops, &self.scroll.item_heights);
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
