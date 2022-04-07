use crate::{ArcYard, layout, Link, render, Trigger, yard};
use crate::layout::LayoutState;
use crate::pod::Pod;
use crate::spot::spot_table::SpotTable;
use crate::yui::layout::ActiveFocus;

pub struct YardPod {
	yard: ArcYard,
	width_height: (i32, i32),
	layout: LayoutState,
	refresh_trigger: Trigger,
	pub spot_table: SpotTable,
}

impl YardPod {
	pub fn new(refresh_trigger: Trigger) -> Self {
		let yard = yard::empty();
		let width_height = (0, 0);
		let layout = layout::run(width_height.1, width_height.0, &yard, &refresh_trigger, &ActiveFocus::default());
		let spot_table = SpotTable::new(layout.max_y, layout.max_x);
		YardPod { yard, width_height, layout, refresh_trigger, spot_table }
	}
	pub fn active_focus(&self) -> &ActiveFocus { &self.layout.active_focus }
	fn set_focus(&mut self, new_focus: ActiveFocus) {
		self.layout.active_focus = new_focus;
		self.refresh_trigger.send(());
	}
}

impl Pod for YardPod {
	fn set_yard(&mut self, yard: ArcYard) {
		self.yard = yard;
		self.refresh_trigger.send(());
	}
	fn set_width_height(&mut self, width_height: (i32, i32)) { self.width_height = width_height; }
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
	fn layout_and_render(&mut self) -> &SpotTable {
		info!("Pod width,height: {:?}", self.width_height);
		self.layout = layout::run(self.width_height.1, self.width_height.0, &self.yard, &self.refresh_trigger, &self.layout.active_focus);
		self.spot_table = render::run(&self.yard, self.layout.max_x, self.layout.max_y, self.layout.bounds_hold.clone(), self.layout.active_focus.focus_id());
		&self.spot_table
	}
}
