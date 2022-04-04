use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, RecvError, Sender};
use std::thread;

use ncurses::*;

use crate::{layout, render, SenderLink, yard};
use crate::palette::Palette;
use crate::yard::ArcYard;
use crate::yui::bounds::BoundsHold;
use crate::yui::layout::ActiveFocus;

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
	bounds_hold: Rc<RefCell<BoundsHold>>,
}

impl System {
	fn new(screen: Sender<ScreenAction>) -> Self {
		let (max_x, max_y) = (0, 0);
		let (_, bounds) = BoundsHold::init(max_x, max_y);
		let yard = yard::empty();
		let active_focus = ActiveFocus::default();
		System { screen, yard, active_focus, max_x, max_y, bounds_hold: bounds }
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
		info!("Curses width:{}, height:{}", max_x, max_y);
		let refresh_link = SenderLink::new(self.screen.clone(), |_| ScreenAction::ResizeRefresh);
		let result = layout::run(max_y, max_x, &self.yard, refresh_link, &self.active_focus);
		self.active_focus = result.active_focus;
		self.max_x = result.max_x;
		self.max_y = result.max_y;
		self.bounds_hold = result.bounds;
	}
	fn resize_refresh(&mut self) {
		self.update_bounds();

		let focus_id = self.active_focus.focus_id();
		let bounds_hold = self.bounds_hold.clone();
		let draw_pad = render::run(self.yard.clone(), self.max_x, self.max_y, bounds_hold, focus_id);

		info!("Screen width: {}, height: {}", self.max_x, self.max_y);
		let palette = Palette::new();
		draw_pad.each(|y, x, front| {
			if let Some((glyph, attr)) = palette.to_glyph_attr(front) {
				if y == 0 && x == 0 {
					info!("Top left: {}, attr: {}", glyph, attr);
				} else if y == 0 && x == (self.max_x - 1) {
					info!("Top right: {}, attr: {}", glyph, attr);
				} else if y == (self.max_y - 1) && x == (self.max_x - 1) {
					info!("Bottom right: {}, attr: {}", glyph, attr);
				} else if y == (self.max_y - 1) && x == 0 {
					info!("Bottom left: {}, attr:{}", glyph, attr);
				} else if (y - self.max_y / 2).abs() < 2 && (x - self.max_x / 2).abs() < 2 {
					info!("Center ({},{}): {}, attr:{}", x, y, glyph, attr);
				}
				attrset(attr);
				mvaddstr(y as i32, x as i32, glyph);
			}
		});
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

