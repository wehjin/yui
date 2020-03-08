extern crate ncurses;

use std::cell::RefCell;
use std::rc::Rc;

use ncurses::*;

use yui::*;

use crate::yui::bounds::{Bounds, BoundsHold};
use crate::yui::button::button_yard;
use crate::yui::empty::empty_yard;
use crate::yui::fill::fill_yard;
use crate::yui::label::label_yard;
use crate::yui::layout::LayoutContextImpl;
use crate::yui::palette::{FillColor, Palette, StrokeColor};

mod yui;

fn main() {
	let header =
		label_yard("Buttons", StrokeColor::PrimaryBody, Cling::Custom { x: 0.0, y: 0.0 }).pad(1)
			.before(fill_yard(FillColor::Primary));


	let button_pole =
		button_yard("Enabled").confine_height(1, Cling::CenterMiddle)
			.pack_top(1, empty_yard())
			.pack_top(1, button_yard("Focused"));
	let content =
		button_pole.confine(32, 3, Cling::CenterMiddle).pad(1)
			.before(fill_yard(FillColor::Background));
	let yard = content.pack_top(3, header);

	initscr();
	if !has_colors() {
		endwin();
		println!("Your terminal does not support color");
		std::process::exit(1);
	}
	raw();
	keypad(stdscr(), true);
	noecho();
	curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
	clear();

	let palette = Palette::new();

	let mut max_x = 0;
	let mut max_y = 0;
	getmaxyx(stdscr(), &mut max_y, &mut max_x);
	let (init_index, init_hold) = BoundsHold::init(max_x, max_y);
	yard.layout(&mut LayoutContextImpl::new(init_index, init_hold.clone()));

	let mut ctx = CursesRenderContext::new(
		max_y,
		max_x,
		init_hold.clone(),
		&palette,
	);
	for row in 0..max_y {
		ctx.row = row;
		for col in 0..max_x {
			ctx.col = col;
			yard.render(&ctx);
			ctx.publish();
		}
	}

	refresh();
	getch();
	use_default_colors();
	endwin();
}


struct CursesRenderContext<'a> {
	row: i32,
	col: i32,
	bounds_hold: Rc<RefCell<BoundsHold>>,
	cols: i32,
	spots: Vec<RefCell<SpotStack<'a>>>,
}

impl<'a> CursesRenderContext<'a> {
	fn new(
		rows: i32,
		cols: i32,
		bounds_hold: Rc<RefCell<BoundsHold>>,
		palette: &'a Palette,
	) -> Self {
		let origin_stack = SpotStack::new(palette);
		CursesRenderContext {
			row: 0,
			col: 0,
			bounds_hold,
			cols,
			spots: vec![origin_stack; (rows * cols) as usize].into_iter().map(|it| RefCell::new(it)).collect(),
		}
	}

	fn spot_stack(&self) -> &RefCell<SpotStack<'a>> {
		let index = self.row * self.cols + self.col;
		&self.spots[index as usize]
	}

	fn publish(&self) {
		mv(self.row as i32, self.col as i32);
		let stack = self.spot_stack();
		let (color_pair_index, glyph) = stack.borrow().color_pair_index_and_glyph();
		attrset(COLOR_PAIR(color_pair_index));
		addch(glyph as chtype);
	}
}

impl<'a> RenderContext for CursesRenderContext<'a> {
	fn spot(&self) -> (i32, i32) {
		(self.row as i32, self.col as i32)
	}

	fn yard_bounds(&self, yard_id: i32) -> Bounds {
		self.bounds_hold.borrow().yard_bounds(yard_id).to_owned()
	}

	fn set_fill(&self, color: FillColor, z: i32) {
		self.spot_stack().borrow_mut().set_fill(color, z);
	}

	fn set_glyph(&self, glyph: char, color: StrokeColor, z: i32) {
		self.spot_stack().borrow_mut().set_stroke(glyph, color, z);
	}
}


#[derive(Copy, Clone, Debug)]
struct SpotStack<'a> {
	fill_color: FillColor,
	fill_z: i32,
	stroke_type: Option<(char, StrokeColor)>,
	stroke_z: i32,
	palette: &'a Palette,
}

impl<'a> SpotStack<'a> {
	fn new(palette: &'a Palette) -> Self {
		SpotStack {
			fill_color: FillColor::Background,
			fill_z: i32::max_value(),
			stroke_type: Option::None,
			stroke_z: i32::max_value(),
			palette,
		}
	}

	fn set_fill(&mut self, color: FillColor, z: i32) {
		if z <= self.fill_z {
			self.fill_z = z;
			self.fill_color = color;
		}
	}

	fn set_stroke(&mut self, glyph: char, color: StrokeColor, z: i32) {
		if z <= self.stroke_z {
			self.stroke_z = z;
			self.stroke_type = Option::Some((glyph, color))
		}
	}

	fn color_pair_index_and_glyph(&self) -> (i16, char) {
		let fill_color = self.fill_color;
		let (glyph, stroke_color) = match self.stroke_type {
			None => (' ', StrokeColor::Body),
			Some((glyph, color)) => if self.stroke_z <= self.fill_z {
				(glyph, color)
			} else {
				(' ', StrokeColor::Body)
			},
		};
		let color_pair = self.palette.color_pair_index(stroke_color, fill_color);
		(color_pair, glyph)
	}
}
