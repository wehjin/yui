use std::ops::Deref;
use std::sync::{Arc, RwLock};

use crate::{ArcYard, Before, Cling, Focus, FocusType, Pack, Place, render_submit, RenderContext, yard};
use crate::layout::LayoutContext;
use crate::palette::{FillColor, StrokeColor};
use crate::yard::{ignore_touch, Yard, YardOption};

pub use self::tab::*;

mod tab;

pub fn tabbar(tabs: &[impl Tab], selected_index: usize, on_select: impl Fn(usize) + Send + Sync + 'static) -> ArcYard {
	let selected_index = Arc::new(RwLock::new(selected_index));
	let on_select =
		Arc::new(on_select);
	let tabs: Vec<(i32, ArcYard)> = tabs.iter().enumerate().map(|(index, tab)| {
		let id = tab.uid();
		let label = tab.label();
		let tab_width = (label.chars().count() + 2 * 2) as i32;
		let tab_selected_index = selected_index.clone();
		let tab_on_select = on_select.clone();
		let tab_yard = tab_yard(id, label, index, tab_selected_index.clone(), move || {
			let old_index = *(tab_selected_index.read().unwrap());
			if index != old_index {
				*(tab_selected_index.write().unwrap()) = index;
				(*tab_on_select)(index)
			}
		});
		(tab_width, tab_yard)
	}).collect();
	let (width, bar) = tabs.into_iter()
		.fold((0, yard::empty()), |(bar_width, bar), (width, tab)| {
			(bar_width + width, bar.pack_right(width, tab))
		});
	let centered_bar = bar.place_center(width);
	let fill = yard::fill(FillColor::Primary);
	centered_bar.before(fill)
}


fn tab_yard(id: i32, label: &str, index: usize, selected: Arc<RwLock<usize>>, select: impl Fn() + Send + Sync + 'static) -> ArcYard {
	let label = yard::label(label, StrokeColor::EnabledOnPrimary, Cling::Center);
	let underline = yard::glyph(StrokeColor::EnabledOnPrimary, move || {
		let selected_index = *selected.read().unwrap();
		if selected_index == index { '_' } else { '\0' }
	});
	let content = yard::empty().pack_bottom(1, label).pack_bottom(1, underline);
	let is_pressed = Arc::new(RwLock::new(false));
	let select = Arc::new(select);
	Arc::new(TabYard { id, content, is_pressed, select })
}

struct TabYard {
	id: i32,
	content: ArcYard,
	is_pressed: Arc<RwLock<bool>>,
	select: Arc<dyn Fn() + Send + Sync>,
}

impl TabYard {
	fn is_pressed(&self) -> bool {
		let is_pressed = self.is_pressed.read().unwrap().deref().to_owned();
		is_pressed
	}
}

impl Yard for TabYard {
	fn id(&self) -> i32 { self.id }
	fn update(&self, _option: YardOption) {}
	fn layout(&self, ctx: &mut LayoutContext) -> usize {
		let (bounds_id, bounds) = ctx.edge_bounds();
		self.content.layout(ctx);
		ctx.set_yard_bounds(self.id(), bounds_id);

		let is_pressed = self.is_pressed.clone();
		let select = self.select.clone();
		ctx.add_focus(Focus {
			yard_id: self.id(),
			focus_type: FocusType::Submit,
			bounds,
			action_block: Arc::new(move |ctx| {
				render_submit(&is_pressed, ctx, &ignore_touch());
				(*select)();
			}),
		});
		bounds_id
	}

	fn render(&self, ctx: &dyn RenderContext) {
		let (row, col) = ctx.spot();
		let bounds = ctx.yard_bounds(self.id);
		if bounds.intersects(row, col) {
			let fill_color = if ctx.focus_id() == self.id() {
				if self.is_pressed() {
					FillColor::PrimaryWithPress
				} else {
					FillColor::PrimaryWithFocus
				}
			} else {
				FillColor::Primary
			};
			ctx.set_fill(fill_color, bounds.z);
			self.content.render(ctx);
		}
	}
}
