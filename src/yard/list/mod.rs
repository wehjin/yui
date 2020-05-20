use std::sync::{Arc, RwLock};

use crate::{ArcYard, Focus, FocusMotion, FocusMotionFuture, FocusType, MultiLayout, palette, RenderContext};
use crate::yard::{Yard, YardOption};
use crate::yard::list::nexus::Nexus;
use crate::yui::bounds::Bounds;
use crate::yui::layout::LayoutContext;

pub fn list(id: i32, items: Vec<(u8, ArcYard)>) -> ArcYard {
	let (item_tops, item_heights, sum_heights, yards) =
		items.into_iter().fold(
			(Vec::new(), Vec::new(), 0, Vec::new()),
			|(mut item_tops, mut heights, sum_heights, mut yards), (height, yard)| {
				let height = height as i32;
				item_tops.push(sum_heights);
				heights.push(height);
				yards.push(yard);
				(item_tops, heights, sum_heights + height, yards)
			},
		);
	Arc::new(ListYard { id, item_heights, item_tops, sum_heights, yards, nexus: Arc::new(RwLock::new(Nexus::new())) })
}

struct ListYard {
	id: i32,
	item_tops: Vec<i32>,
	item_heights: Vec<i32>,
	sum_heights: i32,
	yards: Vec<ArcYard>,
	nexus: Arc<RwLock<Nexus>>,
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
		let focus_index = self.nexus.read().unwrap().item_index();
		let (_bounds_id, bounds) = ctx.edge_bounds();
		let mut focus = None;
		let final_bounds_id = {
			let mut multi_layout = MultiLayout::new(ctx);
			for layout_item in self.layout_items(&bounds) {
				if layout_item.index == focus_index {
					focus = Some(self.create_focus(&bounds))
				}
				multi_layout.layout(&layout_item.yard, &layout_item.bounds);
			}
			multi_layout.finish()
		};
		if let Some(focus) = focus {
			ctx.add_focus(focus)
		}
		ctx.set_yard_bounds(self.id(), final_bounds_id);
		final_bounds_id
	}

	fn render(&self, ctx: &dyn RenderContext) {
		let (row, col) = ctx.spot();
		let bounds = ctx.yard_bounds(self.id);
		if bounds.intersects(row, col) {
			let focus_index = if ctx.focus_id() == self.id {
				Some(self.nexus.read().unwrap().item_index())
			} else {
				None
			};
			for layout_item in self.layout_items(&bounds) {
				layout_item.yard.render(ctx);
				if Some(layout_item.index) == focus_index && layout_item.bounds.intersects(row, col) {
					ctx.set_fill(palette::FillColor::BackgroundWithFocus, layout_item.bounds.z)
				}
			}
		}
	}
}


mod nexus;

impl ListYard {
	fn create_focus(&self, bounds: &Bounds) -> Focus {
		let nexus = self.nexus.clone();
		let item_heights = self.item_heights.to_vec();
		let focus = Focus {
			yard_id: self.id,
			focus_type: FocusType::Edit(Arc::new(move |focus_motion| {
				let new_nexus = match focus_motion {
					FocusMotion::Left | FocusMotion::Right => None,
					FocusMotion::Up => nexus.read().unwrap().up(&item_heights),
					FocusMotion::Down => nexus.read().unwrap().down(&item_heights),
				};
				match new_nexus {
					None => FocusMotionFuture::Default,
					Some(new_nexus) => {
						*nexus.write().unwrap() = new_nexus;
						FocusMotionFuture::Skip
					}
				}
			})),
			bounds: bounds.to_owned(),
			action_block: Arc::new(|_ctx| {}),
		};
		focus
	}

	fn layout_items(&self, bounds: &Bounds) -> Vec<LayoutItem> {
		let nexus = self.nexus.read().unwrap();
		let pivot_row = nexus.pivot_row(bounds.height(), bounds.top, self.sum_heights);
		let pivot_pos = nexus.pivot_pos();
		let mut layout_items = Vec::new();
		let mut next_index = Some(0);
		while next_index.is_some() {
			let index = next_index.unwrap();
			next_index = if index >= self.item_heights.len() {
				None
			} else {
				let item_bounds = nexus.item_bounds(index, bounds, pivot_row, pivot_pos, &self.item_tops, &self.item_heights);
				let (next, keep) = if item_bounds.bottom < bounds.top {
					// Full underflow
					(Some(index + 1), false)
				} else if item_bounds.top < bounds.top {
					// Partial underflow and possibly overflow
					info!("PARTIAL UNDERFLOW, MAYBE OVERFLOW");
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
