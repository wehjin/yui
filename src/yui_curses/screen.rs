use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

use ncurses::*;

use crate::yard::ArcYard;
use crate::yui::bounds::{Bounds, BoundsHold};
use crate::yui::empty::empty_yard;
use crate::yui::layout::{ActiveFocus, LayoutContext};
use crate::yui::palette::{FillColor, Palette, StrokeColor};
use crate::yui::RenderContext;

#[derive(Clone)]
pub(crate) struct CursesScreen {}

impl CursesScreen {
	pub(crate) fn start() -> Sender<ScreenAction> {
		let (tx, rx): (Sender<ScreenAction>, Receiver<ScreenAction>) = mpsc::channel();
		curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
		clear();
		let loop_tx = tx.clone();
		thread::spawn(move || {
			let mut active_focus: ActiveFocus = Default::default();
			let mut done = false;
			let mut yard = empty_yard();
			while !done {
				let action = rx.recv().unwrap();
				match action {
					ScreenAction::SetYard(set_yard) => {
						yard = set_yard;
						loop_tx.send(ScreenAction::ResizeRefresh).unwrap();
					}
					ScreenAction::Space => {
						let screen = loop_tx.clone();
						active_focus.insert_space(move || screen.send(ScreenAction::ResizeRefresh).unwrap());
					}
					ScreenAction::AsciiChar(char) => {
						let screen = loop_tx.clone();
						active_focus.insert_char(char, move || {
							screen.send(ScreenAction::ResizeRefresh).unwrap()
						});
					}
					ScreenAction::FocusUp => {
						active_focus = active_focus.move_up();
						loop_tx.send(ScreenAction::ResizeRefresh).unwrap();
					}
					ScreenAction::FocusDown => {
						active_focus = active_focus.move_down();
						loop_tx.send(ScreenAction::ResizeRefresh).unwrap();
					}
					ScreenAction::FocusLeft => {
						active_focus = active_focus.move_left();
						loop_tx.send(ScreenAction::ResizeRefresh).unwrap();
					}
					ScreenAction::FocusRight => {
						active_focus = active_focus.move_right();
						loop_tx.send(ScreenAction::ResizeRefresh).unwrap();
					}
					ScreenAction::ResizeRefresh => {
						let (max_x, max_y) = Self::size();
						let (init_index, init_hold) = BoundsHold::init(max_x, max_y);
						let mut layout_ctx = LayoutContext::new(init_index, init_hold.clone());
						yard.layout(&mut layout_ctx);
						active_focus = layout_ctx.pop_active_focus(&active_focus);
						let palette = Palette::new();
						let mut ctx = CursesRenderContext::new(
							max_y,
							max_x,
							init_hold.clone(),
							&palette,
							active_focus.focus_id(),
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
		tx
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
		let (color_pair_index, glyph, darken) = stack.borrow().color_details();
		let color_attr = COLOR_PAIR(color_pair_index);
		let attr = if darken {
			color_attr | A_DIM()
		} else {
			color_attr
		};
		attrset(attr);
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

	fn set_dark(&self, z: i32) {
		self.spot_stack().borrow_mut().set_dark(z);
	}
}


#[derive(Copy, Clone, Debug)]
struct SpotStack<'a> {
	fill_color: FillColor,
	fill_z: i32,
	stroke_type: Option<(char, StrokeColor)>,
	stroke_z: i32,
	dark_z: i32,
	palette: &'a Palette,
}

impl<'a> SpotStack<'a> {
	fn new(palette: &'a Palette) -> Self {
		SpotStack {
			fill_color: FillColor::Background,
			fill_z: i32::max_value(),
			stroke_type: Option::None,
			stroke_z: i32::max_value(),
			dark_z: i32::max_value(),
			palette,
		}
	}

	fn set_dark(&mut self, z: i32) {
		if z <= self.dark_z {
			self.dark_z = z;
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

	fn color_details(&self) -> (i16, char, bool) {
		let fill_color = self.fill_color;
		let (glyph, stroke_color) = match self.stroke_type {
			None => (' ', StrokeColor::BodyOnBackground),
			Some((glyph, color)) =>
				if self.stroke_z <= self.fill_z {
					(glyph, color)
				} else {
					(' ', StrokeColor::BodyOnBackground)
				},
		};
		let darken = self.dark_z < self.fill_z;
		let color_pair = self.palette.color_pair_index(stroke_color, fill_color, darken);
		(color_pair, glyph, darken)
	}
}

#[derive(Clone)]
pub(crate) enum ScreenAction {
	Close,
	ResizeRefresh,
	FocusUp,
	FocusDown,
	FocusLeft,
	FocusRight,
	Space,
	SetYard(ArcYard),
	AsciiChar(char),
}

