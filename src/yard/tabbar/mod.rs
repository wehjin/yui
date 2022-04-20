use std::sync::Arc;

use crate::{ArcYard, Before, Bounds, Cling, DrawPad, Focus, FocusType, Link, Pack, Place, SenderLink, SyncLink, yard};
use crate::layout::LayoutContext;
use crate::palette::{FillColor, FillGrade, StrokeColor};
use crate::yard::{Yard, YardOption};

pub use self::tab::*;

mod tab;

#[derive(Clone)]
pub struct TabItem {
	id: i32,
	label: String,
}

impl TabItem {
	pub fn new(id: i32, label: &str) -> Self { TabItem { id, label: label.to_string() } }
}

#[derive(Clone)]
pub struct TabBar {
	items: Vec<TabItem>,
	on_select: SenderLink<usize>,
	selected_index: usize,
}

impl TabBar {
	pub fn new(items: Vec<TabItem>, selected_index: usize, on_select: SenderLink<usize>) -> Self {
		TabBar { items, selected_index, on_select }
	}
}

pub fn tabbar(tab_bar: &TabBar) -> ArcYard {
	let tabs: Vec<(i32, ArcYard)> = tab_bar.items.iter().enumerate().map({
		let on_select = tab_bar.on_select.clone();
		move |(index, tab)| {
			let id = tab.id;
			let label = tab.label.to_string();
			let tab_width = (label.chars().count() + 2 * 2) as i32;
			let tab_on_select = on_select.clone().map(move |_| index);
			let tab_yard = tab_yard(id, &label, index, tab_bar.selected_index, tab_on_select);
			(tab_width, tab_yard)
		}
	}).collect();
	let (width, bar) = tabs.into_iter()
		.fold((0, yard::empty()), |(bar_width, bar), (width, tab)| {
			(bar_width + width, bar.pack_right(width, tab))
		});
	let centered_bar = bar.place_center(width);
	let fill = yard::fill(FillColor::Primary, FillGrade::Plain);
	centered_bar.before(fill)
}


fn tab_yard(id: i32, label: &str, index: usize, active_index: usize, select: SenderLink<()>) -> ArcYard {
	let label = yard::label(label, StrokeColor::EnabledOnPrimary, Cling::Center);
	let underline = yard::glyph(StrokeColor::EnabledOnPrimary, move || if index == active_index { '_' } else { '\0' });
	let content = yard::empty().pack_bottom(1, label).pack_bottom(1, underline);
	Arc::new(TabYard { id, content, is_selected: index == active_index, select: select.into() })
}

struct TabYard {
	id: i32,
	content: ArcYard,
	is_selected: bool,
	select: SyncLink<()>,
}

impl Yard for TabYard {
	fn id(&self) -> i32 { self.id }
	fn update(&self, _option: YardOption) {}
	fn layout(&self, ctx: &mut LayoutContext) -> usize {
		let (bounds_id, bounds) = ctx.edge_bounds();
		self.content.layout(ctx);
		ctx.set_yard_bounds(self.id(), bounds_id);

		let on_select = self.select.clone();
		ctx.add_focus(Focus {
			yard_id: self.id(),
			focus_type: FocusType::Submit,
			bounds,
			priority: if self.is_selected { 500 } else { 0 },
			action_block: Arc::new(move |_| { on_select.send(()); }),
		});
		bounds_id
	}

	fn render(&self, bounds: &Bounds, focus_id: i32, pad: &mut dyn DrawPad) -> Option<Vec<(ArcYard, Option<i32>)>> {
		let fill_grade = if focus_id == self.id() { FillGrade::Focus } else { FillGrade::Plain };
		pad.grade(bounds, fill_grade);
		pad.fill(bounds, FillColor::Primary);
		Some(vec![(self.content.clone(), None)])
	}
}

#[cfg(test)]
mod tests {
	use crate::{layout, render, SenderLink, StrokeColor};
	use crate::StrokeColor::{BodyOnBackground, EnabledOnPrimary};
	use crate::yard::tabbar::tab_yard;
	use crate::yui::layout::ActiveFocus;

	#[test]
	fn layout_render() {
		let yard = tab_yard(500, "a", 0, 0, SenderLink::ignore());
		let (max_x, max_y) = (3, 2);
		let layout = layout::run(max_y, max_x, &yard, &SenderLink::ignore(), &ActiveFocus::default());
		let draw_pad = render::run(&yard, layout.max_x, layout.max_y, layout.bounds_hold.clone(), 0);
		let fronts = draw_pad.to_fronts();
		let fills = fronts.iter().flatten()
			.map(|front| front.stroke.clone().unwrap_or((" ".to_string(), StrokeColor::BodyOnBackground)))
			.collect::<Vec<_>>();
		assert_eq!(fills, vec![
			(" ".to_string(), BodyOnBackground), ("a".to_string(), EnabledOnPrimary), (" ".to_string(), BodyOnBackground),
			("_".to_string(), EnabledOnPrimary), ("_".to_string(), EnabledOnPrimary), ("_".to_string(), EnabledOnPrimary),
		])
	}
}
