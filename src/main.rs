extern crate ncurses;

use std::cell::RefCell;
use std::process::exit;
use std::rc::Rc;

use ncurses::*;

use yui::*;

use crate::yui::bounds::{Bounds, BoundsHold};
use crate::yui::layout::LayoutContextImpl;

mod yui;

fn main() {
	let yard = fill::yard().pack_sides(10);

	initscr();
	if !has_colors() {
		endwin();
		println!("Your terminal does not support color");
		exit(1);
	}
	start_color();
	raw();
	keypad(stdscr(), true);
	noecho();
	curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
	clear();

	init_pair(1, COLOR_WHITE, COLOR_BLUE);

	let mut max_x = 0;
	let mut max_y = 0;
	getmaxyx(stdscr(), &mut max_y, &mut max_x);
	let (init_index, init_hold) = BoundsHold::init(max_x, max_y);
	yard.layout(&mut LayoutContextImpl::new(init_index, init_hold.clone()));

	let mut ctx = CursesRenderContext {
		row: 0,
		col: 0,
		bounds_hold: init_hold,
	};
	for row in 0..max_y {
		ctx.row = row;
		for col in 0..max_x {
			ctx.col = col;
			yard.render(&ctx);
		}
	}

	refresh();
	getch();
	endwin();
}

struct CursesRenderContext {
	row: i32,
	col: i32,
	bounds_hold: Rc<RefCell<BoundsHold>>,
}

impl RenderContext for CursesRenderContext {
	fn spot(&self) -> (i32, i32) {
		(self.row, self.col)
	}

	fn yard_bounds(&self, yard_id: i32) -> Bounds {
		self.bounds_hold.borrow().yard_bounds(yard_id).to_owned()
	}

	fn set_fill(&self, row: i32, col: i32) {
		mv(row, col);
		attrset(COLOR_PAIR(1));
		addch(32);
	}
}

