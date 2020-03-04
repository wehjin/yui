extern crate ncurses;

use std::process::exit;

use ncurses::*;

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
}

trait RenderContext {
	fn row(&self) -> i32;
	fn col(&self) -> i32;
	fn set_glyph(&self, row: i32, col: i32);
}

trait Yard {
	fn layout(&mut self, bounds: &Bounds) -> Bounds;
	fn render(&self, ctx: &dyn RenderContext);
}

struct FillYard {
	edge_bounds: Bounds
}

impl FillYard {
	fn new() -> FillYard {
		FillYard { edge_bounds: Bounds::zero() }
	}
}

impl Yard for FillYard {
	fn layout(&mut self, bounds: &Bounds) -> Bounds {
		self.edge_bounds = bounds.clone();
		self.edge_bounds
	}

	fn render(&self, ctx: &dyn RenderContext) {
		let row = ctx.row();
		let col = ctx.col();
		if self.edge_bounds.intersects(row, col) {
			ctx.set_glyph(row, col)
		}
	}
}

fn fill_yard() -> Box<dyn Yard> {
	Box::new(FillYard::new())
}

struct CursesRenderContext {
	row: i32,
	col: i32,
}

impl RenderContext for CursesRenderContext {
	fn row(&self) -> i32 { self.row }
	fn col(&self) -> i32 { self.col }
	fn set_glyph(&self, row: i32, col: i32) {
		mv(row, col);
		attrset(COLOR_PAIR(1));
		addch(32);
	}
}


fn main() {
	let mut yard = fill_yard();
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

	let bounds = Bounds::new(max_x, max_y);
	yard.layout(&bounds);

	let mut ctx = CursesRenderContext { row: 0, col: 0 };
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
