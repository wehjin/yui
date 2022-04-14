use std::collections::{HashMap, HashSet};

pub use branch::*;
pub use path::*;

use crate::{ArcYard, Bounds, layout, Link, render, StoryId, Trigger, yard};
use crate::layout::LayoutState;
use crate::pod::Pod;
use crate::spot::spot_table::SpotTable;
use crate::yui::layout::ActiveFocus;

mod branch;
mod path;

pub struct PodTree {
	root_path: PodPath,
	refresh_trigger: Trigger,
	yard_map: HashMap<StoryId, ArcYard>,
	layout_map: HashMap<PodPath, LayoutState>,
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
		while let Some(path) = paths.pop() {
			let tail_branch = path.last_branch();
			let yard = self.yard_map.get(&tail_branch.story_id).cloned().unwrap_or_else(yard::empty);
			let layout_state = layout::run(tail_branch.bounds.height(), tail_branch.bounds.width(), &yard, &self.refresh_trigger, &self.active_focus);
			let children = layout_state.to_pod_branches().into_iter().map(|child_branch| path.append_branch(child_branch)).collect::<HashSet<_>>();
			self.layout_map.insert(path.clone(), layout_state);
			let old_children = self.children.remove(&path).unwrap_or_else(|| HashSet::new());
			let dropped_children = old_children.difference(&children).cloned().collect::<Vec<_>>();
			let added_children = children.difference(&old_children).cloned().collect::<Vec<_>>();
			if !children.is_empty() {
				self.children.insert(path.clone(), children);
			}
			self.drop_paths(dropped_children);
			paths.extend(added_children);
			altered.push(path.clone());
		}
		// TODO Update the active focus based on all layouts instead of just the root
		self.active_focus = self.layout_map.get(&self.root_path).map(|layout| layout.active_focus.clone()).unwrap_or_else(|| ActiveFocus::default());
		while let Some(path) = altered.pop() {
			let yard = self.yard_map.get(path.last_story_id()).cloned().unwrap_or_else(yard::empty);
			let bounds_hold = self.layout_map.get(&path).expect("layout state").bounds_hold.clone();
			let spot_table = render::run(&yard, path.last_bounds().width(), path.last_bounds().height(), bounds_hold, self.active_focus.focus_id());
			self.spots_map.insert(path, spot_table);
		}
		let mut paths = self.spots_map.keys().cloned().collect::<Vec<_>>();
		paths.sort_by_key(PodPath::len);
		paths.reverse();
		let mut expanded_tables = HashMap::<PodPath, SpotTable>::new();
		for path in paths {
			let unlinked_table = self.spots_map.get(&path).expect("spot table").clone();
			let linked_table = if let Some(children) = self.children.get(&path) {
				let mut sum = unlinked_table;
				for child in children {
					let child_story = child.last_story_id();
					let child_bounds = child.last_bounds();
					let child_table = expanded_tables.get(child).expect("child table");
					let child_z = child_bounds.z;
					let child_depth = child_table.nearest_z().min(0).abs() + 1;
					sum = sum.expand_seam(child_z, child_depth, (child_story, child_bounds));
					sum.insert_seam(child_table, child_z, child_bounds.left, child_bounds.top);
				}
				sum
			} else { unlinked_table };
			expanded_tables.insert(path, linked_table);
		}
		self.linked_table = expanded_tables.get(&self.root_path).cloned().expect("root table");
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
