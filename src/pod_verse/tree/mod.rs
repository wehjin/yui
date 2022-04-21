use std::collections::{HashMap, HashSet};

pub use branch::*;
pub use path::*;

use crate::{ArcYard, Bounds, layout, Link, render, StoryId, Trigger, yard};
use crate::layout::{LayoutState, to_active_focus};
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

	fn child_layout_depth(&self, child_path: &PodPath) -> i32 {
		if let Some(layout) = self.layout_map.get(child_path) {
			(layout.bounds_hold.borrow().nearest_z() - 1).abs()
		} else { 0 }
	}

	fn path_active_focus<'a>(&'a self, path: &PodPath, expanded_focus_map: &'a HashMap<PodPath, ActiveFocus>) -> Option<&'a ActiveFocus> {
		let expanded = expanded_focus_map.get(path);
		if expanded.is_some() {
			expanded
		} else {
			self.layout_map.get(&path).map(|it| &it.active_focus)
		}
	}

	fn layout_paths(&mut self, mut paths: Vec<PodPath>) {
		info!("LAYOUT PATHS({}): {:?}", paths.len(), paths);
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
		{
			let all_paths_longest_first = {
				let mut paths = self.layout_map.keys().cloned().collect::<Vec<_>>();
				paths.sort_by_key(|it| it.len());
				paths.reverse();
				paths
			};
			trace!("Pod paths longest first({}): {:?}", all_paths_longest_first.len(),&all_paths_longest_first);
			let mut expanded_focus_map: HashMap<PodPath, ActiveFocus> = HashMap::new();
			for parent_path in all_paths_longest_first {
				let mut parent_focus = self.path_active_focus(&parent_path, &expanded_focus_map).cloned().unwrap_or_else(|| ActiveFocus::default());
				trace!("Parent focus BEFORE expansion({:?}): {:?}",parent_path.last_story_id(), &parent_focus);
				if let Some(direct_children) = self.children.get(&parent_path) {
					for child_path in direct_children {
						let child_bounds = child_path.last_bounds();
						let child_depth = self.child_layout_depth(child_path);
						parent_focus.expand_seam(child_bounds.z, child_depth);
						if let Some(child_active_focus) = self.path_active_focus(child_path, &expanded_focus_map) {
							trace!("CHILD focus for insertion({:?}: {:?}", child_path.last_story_id(), child_active_focus);
							parent_focus.insert_seam(child_active_focus, child_bounds.z, child_bounds.left, child_bounds.top);
						}
					}
				};
				trace!("Parent focus AFTER expansion: {:?}", &parent_focus);
				expanded_focus_map.insert(parent_path, parent_focus);
			}
			self.focus_map = expanded_focus_map;
			let root_active = self.focus_map.get(&self.root_path).cloned().unwrap_or_else(|| ActiveFocus::default());
			self.active_focus = to_active_focus(&self.active_focus, root_active.to_foci());
			trace!("NEW ACTIVE FOCUS: {:?}", &self.active_focus);
		}
		// Render altered pods.
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
		let fallback_spot_table = SpotTable::new(0, 0);
		for path in paths {
			let unlinked_table = self.spots_map.get(&path).expect("spot table").clone();
			let linked_table = if let Some(children) = self.children.get(&path) {
				let mut sum = unlinked_table;
				for child in children {
					let child_story = child.last_story_id();
					let child_bounds = child.last_bounds();
					let child_table = expanded_tables.get(child)
						.unwrap_or_else(|| {
							warn!("Not expanded table for child pod: {:?}", child);
							&fallback_spot_table
						});
					let child_depth = self.child_layout_depth(child);
					sum = sum.expand_seam(child_bounds.z, child_depth, (child_story, child_bounds));
					sum.insert_seam(child_table, child_bounds.z, child_bounds.left, child_bounds.top);
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
