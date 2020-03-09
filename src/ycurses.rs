use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::mpsc;
use std::thread;

use ncurses::*;

use crate::yui::{RenderContext, Yard};
use crate::yui::bounds::{Bounds, BoundsHold};
use crate::yui::layout::LayoutContext;
use crate::yui::palette::{FillColor, Palette, StrokeColor};

pub struct CursesScreen {
	tx: Sender<ScreenAction>
}

impl CursesScreen {
	pub fn start(gen_yard: impl FnOnce() -> Rc<dyn Yard> + std::marker::Send + 'static) -> CursesScreen {
		let (tx, rx): (Sender<ScreenAction>, Receiver<ScreenAction>) = mpsc::channel();
		curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
		clear();
		thread::spawn(move || {
			let yard = gen_yard();
			let mut done = false;
			while !done {
				let action = rx.recv().unwrap();
				info!("ACTION: {:?}", action);
				match action {
					ScreenAction::ResizeRefresh => {
						let (max_x, max_y) = Self::size();
						let (init_index, init_hold) = BoundsHold::init(max_x, max_y);
						let mut layout_ctx = LayoutContext::new(init_index, init_hold.clone());
						yard.layout(&mut layout_ctx);
						info!("LayoutContext after layout: {:?}", layout_ctx);
						let focus_id = layout_ctx.focus_id();

						let palette = Palette::new();
						let mut ctx = CursesRenderContext::new(
							max_y,
							max_x,
							init_hold.clone(),
							&palette,
							focus_id,
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
					}
					ScreenAction::Close => { done = true }
				}
			}
		});
		tx.send(ScreenAction::ResizeRefresh).unwrap();
		CursesScreen { tx }
	}

	pub fn resize_and_refresh(&self) {
		self.tx.send(ScreenAction::ResizeRefresh).unwrap();
	}

	pub fn close(&self) {
		self.tx.send(ScreenAction::Close).unwrap();
	}

	fn size() -> (i32, i32) {
		let mut max_x = 0;
		let mut max_y = 0;
		getmaxyx(stdscr(), &mut max_y, &mut max_x);
		(max_x, max_y)
	}
}


struct CursesRenderContext<'a> {
	row: i32,
	col: i32,
	bounds_hold: Rc<RefCell<BoundsHold>>,
	cols: i32,
	spots: Vec<RefCell<SpotStack<'a>>>,
	focus_id: i32,
}

impl<'a> CursesRenderContext<'a> {
	fn new(
		rows: i32,
		cols: i32,
		bounds_hold: Rc<RefCell<BoundsHold>>,
		palette: &'a Palette,
		focus_id: i32,
	) -> Self {
		let origin_stack = SpotStack::new(palette);
		CursesRenderContext {
			row: 0,
			col: 0,
			bounds_hold,
			cols,
			spots: vec![origin_stack; (rows * cols) as usize].into_iter().map(|it| RefCell::new(it)).collect(),
			focus_id,
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
	fn focus_id(&self) -> i32 {
		self.focus_id
	}

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

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum ScreenAction {
	Close,
	ResizeRefresh,
}

pub const KEY_EOT: i32 = 4;

