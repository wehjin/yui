use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, RecvError, SyncSender};
use std::thread;

use ncurses::*;

pub(crate) use spot_stack::*;

use crate::palette::{FillColor, FillGrade, Palette, StrokeColor};
use crate::yard;
use crate::yard::ArcYard;
use crate::yui::bounds::{Bounds, BoundsHold};
use crate::yui::layout::{ActiveFocus, LayoutContext};
use crate::yui::RenderContext;

mod spot_stack;

#[derive(Clone)]
pub(crate) struct CursesScreen {}

impl CursesScreen {
	pub(crate) fn start() -> SyncSender<ScreenAction> {
		let (tx, rx) = mpsc::sync_channel(64);
		curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
		clear();
		let loop_tx = tx.clone();
		thread::spawn(move || {
			let mut active_focus: ActiveFocus = Default::default();
			let mut yard = yard::empty();
			loop {
				let action = match next_screen_action(&rx, &loop_tx) {
					Ok(action) => action,
					Err(_) => break
				};
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
						let screen = loop_tx.clone();
						let mut layout_ctx = LayoutContext::new(
							init_index,
							init_hold.clone(),
							move || screen.send(ScreenAction::ResizeRefresh).unwrap(),
						);
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
					ScreenAction::Close => break
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

fn next_screen_action(rx: &Receiver<ScreenAction>, tx: &SyncSender<ScreenAction>) -> Result<ScreenAction, RecvError> {
	let mut first = rx.recv()?;
	if let ScreenAction::ResizeRefresh = &first {
		let mut done_trying_second = false;
		while !done_trying_second {
			match rx.try_recv() {
				Err(_) => {
					done_trying_second = true
				}
				Ok(second) => match second {
					ScreenAction::ResizeRefresh => {}
					_ => {
						let last = first;
						first = second;
						tx.send(last).unwrap();
						done_trying_second = true
					}
				},
			}
		}
	}
	Ok(first)
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
		if glyph == ' ' {
			addch(glyph as chtype);
		} else {
			addstr(&glyph.to_string());
		}
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
	fn set_fill_grade(&self, fill_grade: FillGrade, z: i32) {
		self.spot_stack().borrow_mut().set_fill_grade(fill_grade, z);
	}
	fn set_glyph(&self, glyph: char, color: StrokeColor, z: i32) {
		self.spot_stack().borrow_mut().set_stroke(glyph, color, z);
	}
	fn set_dark(&self, z: i32) {
		self.spot_stack().borrow_mut().set_dark(z);
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

