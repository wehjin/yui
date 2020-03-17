#[macro_use]
extern crate log;
extern crate ncurses;
extern crate simplelog;

use std::fs::File;
use std::sync::Arc;

use log::LevelFilter;
use simplelog::{Config, WriteLogger};

use yui::*;

use crate::yui::button::button_yard;
use crate::yui::empty::empty_yard;
use crate::yui::fill::fill_yard;
use crate::yui::glyph::glyph_yard;
use crate::yui::label::label_yard;
use crate::yui::layout::LayoutContext;
use crate::yui::palette::{FillColor, StrokeColor};
use crate::yui_curses::Projector;

mod yui;
mod yui_curses;

fn main() {
	WriteLogger::init(LevelFilter::Info, Config::default(), File::create("yui.log").unwrap()).unwrap();
	Projector::run_blocking(|ctx| {
		let header_row = app_bar();
		let focused_button = button_yard("Focused");
		let enabled_button = button_yard("Enabled");
		let button_pole = enabled_button
			.pack_top(1, empty_yard())
			.pack_top(1, focused_button);

		let content_row = button_pole.confine(32, 3, Cling::CenterMiddle)
			.pad(1)
			.before(fill_yard(FillColor::Background));

		let yard = content_row
			.pack_top(3, tabbar_yard())
			.pack_top(3, header_row);
		ctx.set_yard(yard);
	});
}

fn tabbar_yard() -> ArcYard {
	let labels = ["Home", "Merchandise", "About Us"];
	let tabs: Vec<(i32, ArcYard)> = labels.iter().enumerate().map(|(index, label)| {
		let tab_width = (label.chars().count() + 2 * 2) as i32;
		let tab_type = if index == 0 { TabType::Selected } else { TabType::Enabled };
		let tab = tab_yard(label, tab_type);
		(tab_width, tab)
	}).collect();
	let (width, bar) = tabs.into_iter()
		.fold((0, empty_yard()), |(bar_width, bar), (width, tab)| {
			(bar_width + width, bar.pack_right(width, tab))
		});
	let centered_bar = bar.place_center(width);
	let fill = fill_yard(FillColor::Primary);
	centered_bar.before(fill)
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum TabType {
	Enabled,
	Selected,
}

fn tab_yard(label: &str, tab_type: TabType) -> ArcYard {
	let label = label_yard(label, StrokeColor::PrimaryBody, Cling::CenterMiddle);
	let underline = if tab_type == TabType::Selected {
		glyph_yard('-', StrokeColor::PrimaryBody)
	} else {
		empty_yard()
	};
	let content = empty_yard().pack_bottom(1, label).pack_bottom(1, underline);
	Arc::new(TabYard { id: rand::random(), content })
}

struct TabYard {
	id: i32,
	content: ArcYard,
}

impl Yard for TabYard {
	fn id(&self) -> i32 { self.id }
	fn update(&self, _option: YardOption) {}
	fn layout(&self, ctx: &mut LayoutContext) -> usize {
		let (bounds_id, bounds) = ctx.edge_bounds();
		self.content.layout(ctx);
		ctx.set_yard_bounds(self.id(), bounds_id);
		ctx.add_focus(Focus {
			yard_id: self.id(),
			focus_type: FocusType::Submit,
			bounds,
			action_block: Arc::new(move |_ctx| {}),
		});
		bounds_id
	}

	fn render(&self, ctx: &dyn RenderContext) {
		let (row, col) = ctx.spot();
		let bounds = ctx.yard_bounds(self.id);
		if bounds.intersects(row, col) {
			let fill_color = if ctx.focus_id() == self.id() {
				FillColor::PrimaryWithFocus
			} else {
				FillColor::Primary
			};
			ctx.set_fill(fill_color, bounds.z);
			self.content.render(ctx);
		}
	}
}

fn app_bar() -> ArcYard {
	let tool_bar = label_yard("Components", StrokeColor::PrimaryBody, Cling::Custom { x: 0.0, y: 0.0 });
	let header_row = tool_bar.pad(1).before(fill_yard(FillColor::Primary));
	header_row
}
