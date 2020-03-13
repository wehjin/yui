use std::sync::mpsc::Sender;

use ncurses::*;

use crate::yui_curses::ScreenAction;

pub(crate) struct Keyboard;

impl Keyboard {
	pub(crate) fn read_blocking(screen_tx: Sender<ScreenAction>) {
		raw();
		keypad(stdscr(), true);
		cbreak();
		noecho();
		let mut done = false;
		while !done {
			let ch = getch();
			match ch {
				KEY_DOWN => { screen_tx.send(ScreenAction::FocusDown).unwrap() }
				KEY_UP => { screen_tx.send(ScreenAction::FocusUp).unwrap() }
				KEY_RESIZE => { screen_tx.send(ScreenAction::ResizeRefresh).unwrap() }
				KEY_SPACE => { screen_tx.send(ScreenAction::Space).unwrap() }
				KEY_EOT => {
					screen_tx.send(ScreenAction::Close).unwrap();
					done = true;
				}
				_ => {
					info!("KEY: {}", ch);
				}
			}
		}
		use_default_colors();
		endwin();
	}
}

pub(crate) const KEY_EOT: i32 = 4;
pub(crate) const KEY_SPACE: i32 = 32;
