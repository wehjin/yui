use std::sync::mpsc::{Receiver, Sender};

use ncurses::*;

use crate::yui_curses::screen::ScreenAction;

pub(crate) struct Keyboard;

impl Keyboard {
	pub(crate) fn read_blocking(screen_tx: Sender<ScreenAction>, stop_rx: Receiver<()>) {
		raw();
		keypad(stdscr(), true);
		cbreak();
		noecho();
		timeout(300);
		let mut done = false;
		while !done {
			let ch = getch();
			let action: Option<ScreenAction> = match ch {
				ERR => if stop_rx.try_recv().is_ok() { Some(ScreenAction::Close) } else { None },
				KEY_UP => Some(ScreenAction::FocusUp),
				KEY_DOWN => Some(ScreenAction::FocusDown),
				KEY_LEFT => Some(ScreenAction::FocusLeft),
				KEY_RIGHT => Some(ScreenAction::FocusRight),
				KEY_RESIZE => Some(ScreenAction::ResizeRefresh),
				KEY_EOT => Some(ScreenAction::Close),
				KEY_SPACE => { Some(ScreenAction::Space) }
				KEY_BACKSPACE => { Some(ScreenAction::AsciiChar('\x08')) }
				KEY_DC => { Some(ScreenAction::AsciiChar('\x7f')) }
				_ => {
					let name = keyname(ch).unwrap_or("".to_string());
					let chars: Vec<char> = name.chars().collect();
					match chars.len() {
						1 => Some(ScreenAction::AsciiChar(chars[0])),
						2 if chars[0] == '^' => {
							match chars[1] {
								'?' => Some(ScreenAction::AsciiChar('\x08')),
								_ => {
									info!("UNHANDLED CTRL-KEY: {}, {}", ch, name);
									None
								}
							}
						}
						_ => {
							info!("UNHANDLED KEY: {}, {}", ch, name);
							None
						}
					}
				}
			};
			if let Some(action) = action {
				done = match &action {
					ScreenAction::Close => true,
					_ => false
				};
				screen_tx.send(action).expect("send screen action")
			}
		}
		use_default_colors();
		endwin();
	}
}

pub(crate) const KEY_EOT: i32 = 4;
pub(crate) const KEY_SPACE: i32 = 32;
