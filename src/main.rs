extern crate ncurses;

use std::cell::RefCell;
use std::rc::Rc;

use ncurses::*;

use yui::*;

use crate::yui::bounds::{Bounds, BoundsHold};
use crate::yui::fill::fill_yard;
use crate::yui::glyph::glyph_yard;
use crate::yui::layout::LayoutContextImpl;
use crate::yui::palette::{FillColor, Palette, StrokeColor};

mod yui;

fn main() {
	let yard = glyph_yard('#').pad(1).before(fill_yard());

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

	fn set_fill(&self, z: i32) {
		self.spot_stack().borrow_mut().set_fill(z);
	}

	fn set_glyph(&self, glyph: char, z: i32) {
		self.spot_stack().borrow_mut().set_stroke(glyph, z);
	}
}


#[derive(Copy, Clone, Debug)]
struct SpotStack<'a> {
	fill_type: bool,
	fill_z: i32,
	stroke_type: Option<char>,
	stroke_z: i32,
	palette: &'a Palette,
}

impl<'a> SpotStack<'a> {
	fn new(palette: &'a Palette) -> Self {
		SpotStack {
			fill_type: false,
			fill_z: i32::max_value(),
			stroke_type: Option::None,
			stroke_z: i32::max_value(),
			palette,
		}
	}

	fn set_fill(&mut self, z: i32) {
		if z <= self.fill_z {
			self.fill_z = z;
			self.fill_type = true;
		}
	}

	fn set_stroke(&mut self, glyph: char, z: i32) {
		if z <= self.stroke_z {
			self.stroke_z = z;
			self.stroke_type = Option::Some(glyph)
		}
	}

	fn color_pair_index_and_glyph(&self) -> (i16, char) {
		let color_pair = self.palette.color_pair_index(StrokeColor::Body, FillColor::Background);
		let glyph = match self.stroke_type {
			None => ' ',
			Some(glyph) => {
				match self.fill_type == false || self.stroke_z <= self.fill_z {
					true => glyph,
					false => ' '
				}
			}
		};
		(color_pair, glyph)
	}
}
