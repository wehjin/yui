use std::sync::mpsc;
use std::sync::mpsc::{Receiver, RecvError, RecvTimeoutError, Sender};
use std::thread;
use std::time::Duration;

use ncurses::*;

use crate::palette::Palette;
use crate::pod::Pod;
use crate::pod_verse::PodVerse;
use crate::Sendable;
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

pub fn connect_pod_verse(pod_verse: PodVerse) -> Sender<ScreenAction> {
	curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
	clear();
	let (screen_link, actions_source) = mpsc::channel();
	{
		let screen_link = screen_link.clone();
		thread::spawn(move || {
			let pod = pod_verse.to_main_pod(ScreenAction::ResizeRefresh.into_trigger(&screen_link));
			let mut state = ScreenState::init(Box::new(pod)).expect("ScreenState");
			let mut delay_refresh = false;
			loop {
				let mut repeat_try = false;
				match actions_source.recv_timeout(Duration::from_millis(30)) {
					Ok(action) => {
						repeat_try = true;
						if let ScreenAction::ResizeRefresh = action {
							delay_refresh = true;
						} else {
							match state.update(action) {
								None => break,
								Some(new_state) => {
									state = new_state;
								}
							}
						}
					}
					Err(e) => match e {
						RecvTimeoutError::Timeout => {
							if delay_refresh {
								delay_refresh = false;
								match state.update(ScreenAction::ResizeRefresh) {
									None => break,
									Some(new_state) => {
										state = new_state;
									}
								}
							}
						}
						RecvTimeoutError::Disconnected => break,
					},
				}
				if !repeat_try {
					match actions_source.recv() {
						Ok(action) => {
							if let ScreenAction::ResizeRefresh = action {
								delay_refresh = true
							} else {
								match state.update(action) {
									None => break,
									Some(new_state) => {
										state = new_state;
									}
								}
							}
						}
						Err(_) => break,
					}
				}
			}
		});
	}
	screen_link
}

pub fn connect(pod: impl Pod + Send + 'static) -> Sender<ScreenAction> {
	curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
	clear();
	let mut pod = Box::new(pod);
	let (screen_link, actions_source) = mpsc::channel();
	{
		let screen_link = screen_link.clone();
		thread::Builder::new().name("CursesScreen::start".into()).spawn(move || {
			pod.set_refresh_trigger(ScreenAction::ResizeRefresh.into_trigger(&screen_link));
			let mut next = ScreenState::init(pod);
			while let Some(state) = next {
				let action = next_screen_action(&actions_source, &screen_link).ok();
				next = action.map(|action| state.update(action)).flatten();
			}
		}).expect("spawn");
	}
	screen_link
}


struct ScreenState {
	pod: Box<dyn Pod>,
}

impl ScreenState {
	fn init(pod: Box<dyn Pod>) -> Option<Self> {
		Some(ScreenState { pod })
	}
	fn update(mut self, action: ScreenAction) -> Option<Self> {
		let mut stop = false;
		match action {
			ScreenAction::Close => { stop = true; }
			ScreenAction::ResizeRefresh => {
				self.pod.set_width_height(width_height());
				if let Some(rendering) = self.pod.spot_table() {
					update_screen(&rendering);
				}
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
	erase();
	info!("Update Screen");
	let palette = Palette::new();
	spot_table.each(|y, x, front| {
		if let Some((glyph, attr)) = palette.to_glyph_attr(front) {
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

