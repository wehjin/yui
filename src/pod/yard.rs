use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::mpsc::Sender;

use crate::{ArcYard, layout, Link, render, StoryId, Trigger, yard};
use crate::layout::LayoutState;
use crate::pod::Pod;
use crate::pod_verse::PodVerseAction;
use crate::spot::spot_table::SpotTable;
use crate::yui::layout::ActiveFocus;

pub struct YardPod {
	pod_verse_link: Sender<PodVerseAction>,
	story_id: StoryId,
	width_height: (i32, i32),
	yard: ArcYard,
	layout: LayoutState,
	refresh_trigger: Trigger,
	unlinked_spot_table: SpotTable,
	spot_tables: Rc<RefCell<HashMap<(StoryId, (i32, i32)), SpotTable>>>,
}

impl YardPod {
	pub fn new(pod_verse_link: Sender<PodVerseAction>, refresh_trigger: Trigger, story_id: StoryId, (width, height): (i32, i32), spot_tables: Rc<RefCell<HashMap<(StoryId, (i32, i32)), SpotTable>>>) -> Self {
		YardPod {
			pod_verse_link,
			story_id,
			width_height: (width, height),
			yard: yard::empty(),
			layout: layout::run(height, width, &yard::empty(), &refresh_trigger, &ActiveFocus::default()),
			refresh_trigger,
			unlinked_spot_table: SpotTable::new(height, width),
			spot_tables,
		}
	}
	pub fn active_focus(&self) -> &ActiveFocus { &self.layout.active_focus }
	fn set_focus(&mut self, new_focus: ActiveFocus) {
		self.layout.active_focus = new_focus;
		self.refresh_trigger.send(());
	}
	pub fn link_tables(&mut self) {
		let spot_table = self.unlinked_spot_table.clone();
		let linked_table = spot_table.to_seams().iter()
			.fold(spot_table, |sum, (next_story, next_bounds)| {
				let next_z = next_bounds.z;
				let next_id = (*next_story, (next_bounds.width(), next_bounds.height()));
				let tables = self.spot_tables.borrow();
				if let Some(next_table) = tables.get(&next_id) {
					let depth = next_table.nearest_z().min(0).abs() + 1;
					let mut expanded = sum.expand_seam(next_z, depth, (next_story, next_bounds));
					expanded.insert_seam(next_table, next_z, next_bounds.left, next_bounds.top);
					expanded
				} else { sum }
			});
		let mut tables_map = self.spot_tables.borrow_mut();
		tables_map.insert((self.story_id, self.width_height), linked_table);
		self.pod_verse_link.send(PodVerseAction::SpotTableChanged(self.story_id, self.width_height)).expect("send spot-table-changed");
	}
	fn layout_and_render(&mut self) {
		self.layout = layout::run(self.width_height.1, self.width_height.0, &self.yard, &self.refresh_trigger, &self.layout.active_focus);
		self.pod_verse_link.send(PodVerseAction::SetDependencies(self.story_id, self.layout.to_sub_pods())).expect("send set-dependencies");
		self.unlinked_spot_table = render::run(&self.yard, self.layout.max_x, self.layout.max_y, self.layout.bounds_hold.clone(), self.layout.active_focus.focus_id());
		self.link_tables();
	}
}

impl Pod for YardPod {
	fn set_yard(&mut self, yard: ArcYard) {
		trace!("SET_YARD in YardPod {:?}, wh:{:?}", self.story_id, self.width_height);
		self.yard = yard;
		self.layout_and_render();
	}
	fn set_width_height(&mut self, width_height: (i32, i32)) {
		self.width_height = width_height;
		self.layout_and_render();
	}
	fn focus_up(&mut self) { self.set_focus(self.active_focus().move_up()); }
	fn focus_down(&mut self) { self.set_focus(self.active_focus().move_down()); }
	fn focus_left(&mut self) { self.set_focus(self.active_focus().move_left()); }
	fn focus_right(&mut self) { self.set_focus(self.active_focus().move_right()); }
	fn insert_char(&self, char: char) {
		let refresh_trigger = self.refresh_trigger.clone();
		self.active_focus().insert_char(char, move || { refresh_trigger.send(()); });
	}
	fn insert_space(&self) {
		let refresh_trigger = self.refresh_trigger.clone();
		self.active_focus().insert_space(move || { refresh_trigger.send(()); });
	}
	fn set_refresh_trigger(&mut self, trigger: Trigger) {
		self.refresh_trigger = trigger;
	}

	fn spot_table(&self) -> Option<SpotTable> {
		let pod_id = (self.story_id, self.width_height);
		self.spot_tables.borrow().get(&pod_id).cloned()
	}
}
