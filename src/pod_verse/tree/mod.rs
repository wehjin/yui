use std::collections::{HashMap, HashSet};

pub use branch::*;
pub use path::*;

use crate::{ArcYard, Bounds, layout, Link, render, StoryId, Trigger, yard};
use crate::layout::{LayoutState, to_active_focus};
use crate::pod::Pod;
use crate::pod_verse::tree::linker::{link_focus_regions, link_spot_tables};
use crate::spot::spot_table::SpotTable;
use crate::yui::layout::ActiveFocus;

mod branch;
mod path;
mod linker;

pub struct PodTree {
	root_path: PodPath,
	refresh_trigger: Trigger,
	yard_map: HashMap<StoryId, ArcYard>,
	layout_map: HashMap<PodPath, LayoutState>,
	focus_map: HashMap<PodPath, ActiveFocus>,
	children: HashMap<PodPath, HashSet<PodPath>>,
	spots_map: HashMap<PodPath, SpotTable>,
	active_focus: ActiveFocus,
	linked_table: SpotTable,
}

impl Pod for PodTree {
	fn set_yard(&mut self, yard: ArcYard) {
		self.set_story_yard(self.root_path.last_story_id().clone(), Some(yard));
	}

	fn set_width_height(&mut self, width_height: (i32, i32)) {
		self.set_bounds(Bounds::new(width_height.0, width_height.1));
	}

	fn focus_up(&mut self) {
		self.active_focus = self.active_focus.move_up();
	}

	fn focus_down(&mut self) {
		self.active_focus = self.active_focus.move_down();
	}

	fn focus_left(&mut self) {
		self.active_focus = self.active_focus.move_left();
	}

	fn focus_right(&mut self) {
		self.active_focus = self.active_focus.move_right();
	}

	fn insert_char(&self, char: char) {
		let refresh_trigger = self.refresh_trigger.clone();
		self.active_focus.insert_char(char, move || { refresh_trigger.send(()); });
	}

	fn insert_space(&self) {
		let refresh_trigger = self.refresh_trigger.clone();
		self.active_focus.insert_space(move || { refresh_trigger.send(()); });
	}

	fn set_refresh_trigger(&mut self, trigger: Trigger) {
		self.refresh_trigger = trigger;
	}

	fn spot_table(&self) -> Option<SpotTable> {
		Some(self.to_spot_table())
	}
}

impl PodTree {
	pub fn new(story_id: StoryId, refresh_trigger: Trigger) -> Self {
		let mut tree = PodTree {
			root_path: PodPath::new(story_id, Bounds::new(0, 0)),
			refresh_trigger,
			yard_map: HashMap::new(),
			layout_map: HashMap::new(),
			focus_map: HashMap::new(),
			children: HashMap::new(),
			spots_map: HashMap::new(),
			active_focus: ActiveFocus::default(),
			linked_table: SpotTable::new(0, 0),
		};
		tree.layout_paths(vec![tree.root_path.clone()]);
		tree
	}

	pub fn to_spot_table(&self) -> SpotTable { self.linked_table.clone() }

	pub fn root_path(&self) -> &PodPath { &self.root_path }

	pub fn layout_count(&self) -> usize { self.layout_map.len() }

	pub fn set_bounds(&mut self, bounds: Bounds) {
		let path = PodPath::new(self.root_path.last_branch().story_id, bounds);
		if path != self.root_path {
			self.drop_paths(vec![self.root_path.clone()]);
			self.root_path = path.clone();
			self.layout_paths(vec![path]);
		}
	}

	pub fn set_story_yard(&mut self, story_id: StoryId, yard: Option<ArcYard>) {
		let story_paths = self.layout_map.keys().cloned().filter(|it| it.last_story_id() == &story_id).collect::<Vec<_>>();
		if let Some(yard) = yard {
			self.yard_map.insert(story_id, yard);
			self.layout_paths(story_paths);
		} else {
			self.yard_map.remove(&story_id);
			self.drop_paths(story_paths);
		}
	}

	pub fn redraw(&mut self) {
		self.drop_paths(vec![self.root_path.clone()]);
		self.layout_paths(vec![self.root_path.clone()]);
	}

	fn drop_paths(&mut self, mut paths: Vec<PodPath>) {
		while let Some(path) = paths.pop() {
			self.layout_map.remove(&path);
			self.spots_map.remove(&path);
			if let Some(children) = self.children.remove(&path) {
				paths.extend(children)
			}
		}
	}

	fn layout_paths(&mut self, mut paths: Vec<PodPath>) {
		let mut altered = Vec::new();
		//Layout the supplied paths and their descendents.
		while let Some(path) = paths.pop() {
			let tail_branch = path.last_branch();
			let yard = self.yard_map.get(&tail_branch.story_id).cloned().unwrap_or_else(yard::empty);
			let layout_state = layout::run(tail_branch.bounds.height(), tail_branch.bounds.width(), &yard, &self.active_focus);
			let cur_children = layout_state.to_branches().into_iter().map(|child_branch| path.append_branch(child_branch)).collect::<HashSet<_>>();
			let old_children = self.children.remove(&path).unwrap_or_else(|| HashSet::new());
			self.layout_map.insert(path.clone(), layout_state);
			let dropped_children = old_children.difference(&cur_children).cloned().collect::<Vec<_>>();
			let added_children = cur_children.difference(&old_children).cloned().collect::<Vec<_>>();
			if !cur_children.is_empty() {
				self.children.insert(path.clone(), cur_children);
			}
			self.drop_paths(dropped_children);
			paths.extend(added_children);
			altered.push(path.clone());
		}
		// Next recompute the focus.
		self.focus_map = link_focus_regions(&self.layout_map, &self.children);
		let linked_focus = self.focus_map.get(&self.root_path).cloned().unwrap_or_else(|| ActiveFocus::default());
		self.active_focus = to_active_focus(&self.active_focus, linked_focus.to_foci(), linked_focus.rear_z);

		// Render altered pods.
		while let Some(path) = altered.pop() {
			let yard = self.yard_map.get(path.last_story_id()).cloned().unwrap_or_else(yard::empty);
			let bounds_hold = self.layout_map.get(&path).expect("layout state").bounds_hold.clone();
			let spot_table = render::run(&yard, path.last_bounds().width(), path.last_bounds().height(), bounds_hold, self.active_focus.focus_id());
			self.spots_map.insert(path, spot_table);
		}
		self.linked_table = link_spot_tables(&self.spots_map, &self.children, &&self.root_path);
	}
}

#[cfg(test)]
mod tests {
	use crate::{Bounds, SenderLink, StoryId};
	use crate::pod_verse::tree::{PodPath, PodTree};

	#[test]
	fn pod_tree() {
		let story_id = StoryId::new(0);
		let bounds = Bounds::new(20, 20);
		let mut tree = PodTree::new(story_id, SenderLink::ignore());
		tree.set_bounds(bounds);
		assert_eq!(tree.root_path, PodPath::new(story_id, bounds))
	}
}
