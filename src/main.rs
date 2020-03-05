extern crate ncurses;

use std::cell::RefCell;
use std::ops::Deref;
use std::process::exit;
use std::rc::Rc;

use ncurses::*;

use yui::*;

mod yui;

fn main() {
	let yard0 = FillYard::new();
	let yard = PackYard::new(1, yard0);

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
	let mut init_hold = BoundsHold::new();
	let init_index = init_hold.push_root(max_x, max_y);
	let hold_ref = Rc::new(RefCell::new(init_hold));
	let mut layout_ctx = LayoutContextImpl { current_index: init_index, bounds_hold: hold_ref.clone() };
	yard.layout(&mut layout_ctx);

	let mut ctx = CursesRenderContext {
		row: 0,
		col: 0,
		bounds_hold: hold_ref,
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


struct LayoutContextImpl {
	current_index: usize,
	bounds_hold: Rc<RefCell<BoundsHold>>,
}

impl LayoutContext for LayoutContextImpl {
	fn bounds_hold(&self) -> Rc<RefCell<BoundsHold>> {
		self.bounds_hold.clone()
	}

	fn edge_bounds(&self) -> (usize, Bounds) {
		let bounds_index = self.current_index;
		let bounds = self.bounds_hold.borrow().bounds(bounds_index);
		(bounds_index, bounds)
	}

	fn push_core_bounds(&mut self, bounds: &Bounds) -> usize {
		self.bounds_hold.deref().borrow_mut().push_bounds(bounds)
	}

	fn set_yard_bounds(&mut self, yard_id: i32, bounds_index: usize) {
		self.bounds_hold.deref().borrow_mut().insert_yard_bounds(yard_id, bounds_index);
	}
}


struct PackYard {
	yard_id: i32,
	left_cols: i32,
	right_cols: i32,
	top_rows: i32,
	bottom_rows: i32,
	yard: Box<dyn Yard>,
}

impl PackYard {
	fn new(size: i32, yard: impl Yard + 'static) -> PackYard {
		let cols = size * 2;
		let rows = size;
		PackYard {
			yard_id: rand::random(),
			left_cols: cols,
			right_cols: cols,
			top_rows: rows,
			bottom_rows: rows,
			yard: Box::new(yard),
		}
	}
}

impl Yard for PackYard {
	fn yard_id(&self) -> i32 { self.yard_id }

	fn layout(&self, ctx: &mut dyn LayoutContext) -> usize {
		let (index, bounds) = ctx.edge_bounds();
		let alt_bounds = bounds.pack(self.left_cols, self.right_cols, self.top_rows, self.bottom_rows);
		let alt_index = ctx.push_core_bounds(&alt_bounds);
		let mut alt_ctx = LayoutContextImpl { current_index: alt_index, bounds_hold: ctx.bounds_hold().to_owned() };
		self.yard.layout(&mut alt_ctx);
		// TODO Merge packed bounds into near/far.
		index
	}

	fn render(&self, ctx: &dyn RenderContext) {
		self.yard.render(ctx)
	}
}

struct FillYard {
	yard_id: i32,
}

impl FillYard {
	fn new() -> FillYard {
		let yard_id = rand::random();
		FillYard { yard_id }
	}
}

impl Yard for FillYard {
	fn yard_id(&self) -> i32 {
		self.yard_id
	}

	fn layout(&self, ctx: &mut dyn LayoutContext) -> usize {
		let (bounds_id, _bounds) = ctx.edge_bounds();
		ctx.set_yard_bounds(self.yard_id(), bounds_id);
		bounds_id
	}

	fn render(&self, ctx: &dyn RenderContext) {
		let (row, col) = ctx.spot();
		let bounds = ctx.yard_bounds(self.yard_id);
		if bounds.intersects(row, col) {
			ctx.set_fill(row, col)
		}
	}
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

