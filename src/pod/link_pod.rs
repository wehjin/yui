use std::sync::mpsc::{channel, Sender};

use crate::{ArcYard, Trigger};
use crate::pod::Pod;
use crate::pod_verse::{EditAction, MoveDirection, PodVerseAction};
use crate::spot::spot_table::SpotTable;

pub struct MainPod {
	pod_verse_link: Sender<PodVerseAction>,
}

impl MainPod {
	pub fn new(pod_verse_link: Sender<PodVerseAction>, screen_refresh_trigger: Trigger) -> Self {
		pod_verse_link.send(PodVerseAction::SetScreenRefreshTrigger(screen_refresh_trigger)).expect("set screen-refresh-trigger");
		MainPod { pod_verse_link: pod_verse_link.clone() }
	}
	fn send_edit(&self, edit_action: EditAction, msg: &str) {
		self.pod_verse_link.send(PodVerseAction::Edit(edit_action)).expect(msg);
	}
}

impl Pod for MainPod {
	fn set_yard(&mut self, _yard: ArcYard) {
		unimplemented!()
	}

	fn set_width_height(&mut self, width_height: (i32, i32)) {
		self.pod_verse_link.send(PodVerseAction::SetWidthHeight { width: width_height.0, height: width_height.1 }).expect("set-width-height");
	}

	fn focus_up(&mut self) {
		let action = EditAction::MoveFocus(MoveDirection::Up);
		self.send_edit(action, "focus-up");
	}

	fn focus_down(&mut self) {
		let action = EditAction::MoveFocus(MoveDirection::Down);
		self.send_edit(action, "focus-down");
	}

	fn focus_left(&mut self) {
		let action = EditAction::MoveFocus(MoveDirection::Left);
		self.send_edit(action, "focus-left");
	}

	fn focus_right(&mut self) {
		let action = EditAction::MoveFocus(MoveDirection::Right);
		self.send_edit(action, "focus-right");
	}

	fn insert_char(&self, char: char) {
		let action = EditAction::InsertChar(char);
		self.send_edit(action, "insert-char");
	}

	fn insert_space(&self) {
		let action = EditAction::InsertSpace;
		self.send_edit(action, "insert-space");
	}

	fn set_refresh_trigger(&mut self, trigger: Trigger) {
		self.pod_verse_link.send(PodVerseAction::SetScreenRefreshTrigger(trigger)).expect("send set-screen-refresh-trigger");
	}

	fn spot_table(&self) -> Option<SpotTable> {
		let (sender, receiver) = channel();
		self.pod_verse_link.send(PodVerseAction::SpotTable(sender)).expect("read spot-table");
		receiver.recv().expect("receive spot-table")
	}
}
