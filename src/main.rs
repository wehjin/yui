extern crate ncurses;

use std::collections::HashMap;
use std::process::exit;

use ncurses::*;

fn main() {
	let yard = FillYard::new();

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

	let mut bounds_hold = BoundsHold::new();
	let current_index = bounds_hold.push_bounds(max_x, max_y);
	let mut layout_ctx = LayoutContextImpl { current_index, bounds_hold };
	yard.layout(&mut layout_ctx);

	let post_hold = layout_ctx.release_hold();
	let mut ctx = CursesRenderContext {
		row: 0,
		col: 0,
		bounds_hold: post_hold,
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


#[derive(Copy, Clone)]
struct Bounds {
	right: i32,
	bottom: i32,
	left: i32,
	top: i32,
	near: i32,
	far: i32,
}

impl Bounds {
	fn zero() -> Bounds {
		Bounds { right: 0, bottom: 0, left: 0, top: 0, near: 0, far: 0 }
	}
	fn new(width: i32, height: i32) -> Bounds {
		Bounds { right: width, bottom: height, left: 0, top: 0, near: 0, far: 0 }
	}
	fn intersects(&self, row: i32, col: i32) -> bool {
		row >= self.top && row < self.bottom && col >= self.left && col < self.right
	}
	fn pack(&self, rows: i32, cols: i32) -> Bounds {
		Bounds {
			right: self.right - cols,
			bottom: self.bottom - rows,
			left: self.left + cols,
			top: self.top + rows,
			near: self.near,
			far: self.far,
		}
	}
}

struct BoundsHold {
	holdings: Vec<Bounds>,
	map: HashMap<i32, usize>,
}

impl BoundsHold {
	fn new() -> BoundsHold {
		BoundsHold { holdings: Vec::new(), map: HashMap::new() }
	}

	fn get_bounds(&self, bounds_index: usize) -> &Bounds {
		self.holdings.get(bounds_index).expect(&format!("No bounds at index {}", bounds_index))
	}

	fn push_bounds(&mut self, width: i32, height: i32) -> usize {
		self.holdings.push(Bounds::new(width, height));
		self.holdings.len() - 1
	}

	fn get_yard_bounds(&self, yard_id: i32) -> &Bounds {
		let bounds_index = self.map.get(&yard_id).unwrap().to_owned();
		self.holdings.get(bounds_index).unwrap()
	}

	fn insert_yard_bounds(&mut self, yard_id: i32, bounds_index: usize) {
		self.map.insert(yard_id, bounds_index);
	}
}

trait Yard {
	fn yard_id(&self) -> i32;
	fn layout(&self, ctx: &mut dyn LayoutContext) -> usize;
	fn render(&self, ctx: &dyn RenderContext);
}

trait LayoutContext {
	fn edge_bounds(&self) -> (usize, &Bounds);
	fn set_yard_bounds(&mut self, yard_id: i32, bounds_id: usize);
	fn release_hold(self) -> BoundsHold;
}

struct LayoutContextImpl {
	current_index: usize,
	bounds_hold: BoundsHold,
}

impl LayoutContext for LayoutContextImpl {
	fn edge_bounds(&self) -> (usize, &Bounds) {
		let bounds_index = self.current_index;
		let bounds = self.bounds_hold.get_bounds(bounds_index);
		(bounds_index, bounds)
	}

	fn set_yard_bounds(&mut self, yard_id: i32, bounds_index: usize) {
		self.bounds_hold.insert_yard_bounds(yard_id, bounds_index);
	}

	fn release_hold(self) -> BoundsHold {
		self.bounds_hold
	}
}

trait RenderContext {
	fn spot(&self) -> (i32, i32);
	fn yard_bounds(&self, yard_id: i32) -> &Bounds;
	fn set_fill(&self, row: i32, col: i32);
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
	bounds_hold: BoundsHold,
}

impl RenderContext for CursesRenderContext {
	fn spot(&self) -> (i32, i32) {
		(self.row, self.col)
	}

	fn yard_bounds(&self, yard_id: i32) -> &Bounds {
		self.bounds_hold.get_yard_bounds(yard_id)
	}

	fn set_fill(&self, row: i32, col: i32) {
		mv(row, col);
		attrset(COLOR_PAIR(1));
		addch(32);
	}
}

