use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, RecvError, Sender};
use std::thread;

use ncurses::*;

pub(crate) use spot_stack::*;

use crate::{SenderLink, yard};
use crate::palette::{FillColor, FillGrade, Palette, StrokeColor};
use crate::yard::ArcYard;
use crate::yui::bounds::{Bounds, BoundsHold};
use crate::yui::layout::{ActiveFocus, LayoutContext};
use crate::yui::RenderContext;

mod spot_stack;

fn init() {
	curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
	clear();
}

fn width_height() -> (i32, i32) {
	let mut max_x = 0;
	let mut max_y = 0;
	getmaxyx(stdscr(), &mut max_y, &mut max_x);
	(max_x, max_y)
}

struct System {
	screen: Sender<ScreenAction>,
	yard: ArcYard,
	active_focus: ActiveFocus,
	max_x: i32,
	max_y: i32,
	bounds: Rc<RefCell<BoundsHold>>,
}

impl System {
	fn new(screen: Sender<ScreenAction>) -> Self {
		let (max_x, max_y) = (0, 0);
		let (_, bounds) = BoundsHold::init(max_x, max_y);
		let yard = yard::empty();
		let active_focus = ActiveFocus::default();
		System { screen, yard, active_focus, max_x, max_y, bounds }
	}
	fn set_yard(&mut self, yard: ArcYard) {
		self.yard = yard;
		self.screen.send(ScreenAction::ResizeRefresh).expect("Send ResizeRefresh");
	}
	fn insert_space(&self) {
		let screen = self.screen.clone();
		self.active_focus.insert_space(move || {
			screen.send(ScreenAction::ResizeRefresh).expect("Send ResizeRefresh")
		});
	}
	fn insert_char(&self, char: char) {
		let screen = self.screen.clone();
		self.active_focus.insert_char(char, move || {
			screen.send(ScreenAction::ResizeRefresh).expect("send ResizeRefresh in AsciiChar")
		});
	}
	fn focus_up(&mut self) {
		self.set_focus(self.active_focus.move_up());
	}
	fn set_focus(&mut self, new_focus: ActiveFocus) {
		self.active_focus = new_focus;
		self.screen.send(ScreenAction::ResizeRefresh).expect("send ResizeRefresh in SetFocus");
	}
	fn focus_down(&mut self) {
		self.set_focus(self.active_focus.move_down());
	}
	fn focus_left(&mut self) {
		self.set_focus(self.active_focus.move_left());
	}
	fn focus_right(&mut self) {
		self.set_focus(self.active_focus.move_right());
	}
	fn update_bounds(&mut self) {
		let (max_x, max_y) = width_height();
		let (start_index, bounds) = BoundsHold::init(max_x, max_y);

		let mut layout_ctx = LayoutContext::new(
			start_index,
			bounds.clone(),
			SenderLink::new(self.screen.clone(), |_| ScreenAction::ResizeRefresh),
		);
		self.yard.layout(&mut layout_ctx);
		self.active_focus = layout_ctx.pop_active_focus(&self.active_focus);
		self.max_x = max_x;
		self.max_y = max_y;
		self.bounds = bounds;
	}
	fn resize_refresh(&mut self) {
		self.update_bounds();

		let palette = Palette::new();
		let mut ctx = CursesRenderContext::new(
			self.max_y, self.max_x,
			&palette,
			self.bounds.clone(),
			self.active_focus.focus_id(),
		);
		clear();
		for row in 0..self.max_y {
			ctx.row = row;
			for col in 0..self.max_x {
				ctx.col = col;
				self.yard.render(&ctx);
				ctx.publish();
			}
		}
		refresh();
	}
}

fn run(rx: Receiver<ScreenAction>, screen: Sender<ScreenAction>) {
	let mut system = System::new(screen);
	loop {
		let action = match next_screen_action(&rx, &system.screen) {
			Ok(action) => action,
			Err(_) => break
		};
		match action {
			ScreenAction::SetYard(yard) => system.set_yard(yard),
			ScreenAction::Space => system.insert_space(),
			ScreenAction::AsciiChar(char) => system.insert_char(char),
			ScreenAction::FocusUp => system.focus_up(),
			ScreenAction::FocusDown => system.focus_down(),
			ScreenAction::FocusLeft => system.focus_left(),
			ScreenAction::FocusRight => system.focus_right(),
			ScreenAction::ResizeRefresh => system.resize_refresh(),
			ScreenAction::Close => break
		}
	}
}

pub fn connect() -> Sender<ScreenAction> {
	init();
	let (tx, rx) = mpsc::channel();
	let loop_tx = tx.clone();
	thread::Builder::new().name("CursesScreen::start".to_string()).spawn(move || run(rx, loop_tx)).expect("spawn");
	tx.send(ScreenAction::ResizeRefresh).expect("Send ResizeRefresh");
	tx
}

fn next_screen_action(rx: &Receiver<ScreenAction>, tx: &Sender<ScreenAction>) -> Result<ScreenAction, RecvError> {
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
						tx.send(last).expect("send last screen action");
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
		palette: &'a Palette,
		bounds_hold: Rc<RefCell<BoundsHold>>,
		focus_id: i32,
	) -> Self {
		let origin_stack = SpotStack::new(&palette);
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
		let stack = self.spot_stack().borrow();
		let (color_pair_index, glyph, darken) = stack.spot_details();
		let color_attr = COLOR_PAIR(color_pair_index);
		let attr = if darken {
			color_attr | A_DIM()
		} else {
			color_attr
		};
		attrset(attr);
		if !glyph.is_empty() {
			addstr(glyph);
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
	fn set_glyph(&self, glyph: String, color: StrokeColor, z: i32) {
		self.spot_stack().borrow_mut().set_stroke(glyph, color, z);
	}
	fn set_dark(&self, z: i32) {
		self.spot_stack().borrow_mut().set_dark(z);
	}
}

#[derive(Clone)]
pub enum ScreenAction {
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

