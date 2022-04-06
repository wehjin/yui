use std::sync::mpsc;
use std::sync::mpsc::{Receiver, RecvError, Sender};
use std::thread;

use ncurses::*;

use crate::{Sendable, Trigger};
use crate::palette::Palette;
use crate::pod::Pod;
use crate::pod::yard::YardPod;
use crate::spot::spot_table::SpotTable;
use crate::yard::ArcYard;

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

impl Sendable for ScreenAction {}

pub fn connect() -> Sender<ScreenAction> {
	curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
	clear();
	let (screen_link, actions_source) = mpsc::channel();
	launch_state_update_thread(actions_source, screen_link.clone());
	screen_link
}

fn launch_state_update_thread(actions_source: Receiver<ScreenAction>, screen_link: Sender<ScreenAction>) {
	thread::Builder::new().name("CursesScreen::start".into()).spawn(move || {
		let mut next = ScreenState::init(ScreenAction::ResizeRefresh.into_trigger(&screen_link));
		while let Some(state) = next {
			let action = next_screen_action(&actions_source, &screen_link).ok();
			next = action.map(|action| state.update(action)).flatten();
		}
	}).expect("spawn");
}


struct ScreenState {
	pod: YardPod,
}

impl ScreenState {
	fn init(refresh_trigger: Trigger) -> Option<Self> {
		let pod = YardPod::new(refresh_trigger);
		Some(ScreenState { pod })
	}
	fn update(mut self, action: ScreenAction) -> Option<Self> {
		let mut stop = false;
		match action {
			ScreenAction::Close => { stop = true; }
			ScreenAction::ResizeRefresh => {
				self.pod.set_size(width_height());
				let rendering = self.pod.layout_and_render();
				update_screen(rendering);
			}
			ScreenAction::SetYard(yard) => self.pod.set_yard(yard),
			ScreenAction::Space => self.pod.insert_space(),
			ScreenAction::AsciiChar(char) => self.pod.insert_char(char),
			ScreenAction::FocusUp => self.pod.focus_up(),
			ScreenAction::FocusDown => self.pod.focus_down(),
			ScreenAction::FocusLeft => self.pod.focus_left(),
			ScreenAction::FocusRight => self.pod.focus_right(),
		}
		if stop { None } else { Some(ScreenState { pod: self.pod }) }
	}
}

fn width_height() -> (i32, i32) {
	let mut max_x = 0;
	let mut max_y = 0;
	getmaxyx(stdscr(), &mut max_y, &mut max_x);
	(max_x, max_y)
}

fn update_screen(spot_table: &SpotTable) {
	let palette = Palette::new();
	let (max_x, max_y) = spot_table.width_height();
	spot_table.each(|y, x, front| {
		if let Some((glyph, attr)) = palette.to_glyph_attr(front) {
			if y == 0 && x == 0 {
				info!("Top left: {}, attr: {}", glyph, attr);
			} else if y == 0 && x == (max_x - 1) {
				info!("Top right: {}, attr: {}", glyph, attr);
			} else if y == (max_y - 1) && x == (max_x - 1) {
				info!("Bottom right: {}, attr: {}", glyph, attr);
			} else if y == (max_y - 1) && x == 0 {
				info!("Bottom left: {}, attr:{}", glyph, attr);
			} else if (y - max_y / 2).abs() < 2 && (x - max_x / 2).abs() < 2 {
				info!("Center ({},{}): {}, attr:{}", x, y, glyph, attr);
			}
			attrset(attr);
			mvaddstr(y as i32, x as i32, glyph);
		}
	});
	refresh();
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

