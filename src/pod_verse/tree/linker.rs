use std::collections::{HashMap, HashSet};

use crate::layout::LayoutState;
use crate::pod_verse::tree::PodPath;
use crate::spot::spot_table::SpotTable;
use crate::yui::layout::ActiveFocus;

pub fn link_focus_regions(layout_map: &HashMap<PodPath, LayoutState>, children: &HashMap<PodPath, HashSet<PodPath>>) -> HashMap<PodPath, ActiveFocus> {
	let all_paths_longest_first = {
		let mut paths = layout_map.keys().cloned().collect::<Vec<_>>();
		paths.sort_by_key(|it| it.len());
		paths.reverse();
		paths
	};
	trace!("Pod paths longest first({}): {:?}", all_paths_longest_first.len(),&all_paths_longest_first);
	let mut expanded_focus_map: HashMap<PodPath, ActiveFocus> = HashMap::new();
	for parent_path in all_paths_longest_first {
		let path = &parent_path;
		let mut parent_focus = path_active_focus(&path, &expanded_focus_map, layout_map).cloned().unwrap_or_else(|| ActiveFocus::default());
		trace!("Parent focus BEFORE expansion({:?}): {:?}",parent_path.last_story_id(), &parent_focus);
		if let Some(direct_children) = children.get(&parent_path) {
			for child_path in direct_children {
				let child_bounds = child_path.last_bounds();
				let child_depth = if let Some(layout) = layout_map.get(child_path) {
					(layout.bounds_hold.borrow().nearest_z()).abs() + 1
				} else {
					0
				};
				parent_focus.expand_seam(child_bounds.z, child_depth);
				let path = child_path;
				if let Some(child_active_focus) = path_active_focus(&path, &expanded_focus_map, layout_map) {
					trace!("CHILD focus for insertion({:?}: {:?}", child_path.last_story_id(), child_active_focus);
					parent_focus.insert_seam(child_active_focus, child_bounds.z, child_bounds.left, child_bounds.top);
				}
			}
		};
		trace!("Parent focus AFTER expansion: {:?}", &parent_focus);
		expanded_focus_map.insert(parent_path, parent_focus);
	};
	expanded_focus_map
}

fn path_active_focus<'a>(path: &PodPath, expanded_focus_map: &'a HashMap<PodPath, ActiveFocus>, layouts: &'a HashMap<PodPath, LayoutState>) -> Option<&'a ActiveFocus> {
	let expanded = expanded_focus_map.get(path);
	if expanded.is_some() {
		expanded
	} else {
		layouts.get(&path).map(|it| &it.active_focus)
	}
}


pub fn link_spot_tables(spot_tables: &HashMap<PodPath, SpotTable>, children: &HashMap<PodPath, HashSet<PodPath>>, root_path: &&PodPath) -> SpotTable {
	let mut paths = spot_tables.keys().cloned().collect::<Vec<_>>();
	paths.sort_by_key(PodPath::len);
	paths.reverse();
	let mut expanded_tables = HashMap::<PodPath, SpotTable>::new();
	for path in &paths {
		let unlinked_table = spot_tables.get(path).expect("spot table").clone();
		let linked_table = if let Some(children) = children.get(path) {
			let children_nearest_first = {
				let mut vec = children.iter().collect::<Vec<_>>();
				vec.sort_by_key(|it| it.last_bounds().z);
				vec
			};
			let mut sum = unlinked_table;
			for child in children_nearest_first {
				let child_story = child.last_story_id();
				let insert_bounds = child.last_bounds();
				let child_table = expanded_tables.get(child).expect("child expanded table");
				let child_depth = child_table.nearest_z().abs() + 1;
				let insertion_point = insert_bounds.z;
				sum = sum.expand_seam(insertion_point, child_depth, (child_story, insert_bounds));
				sum.insert_seam(child_table, insertion_point, insert_bounds.left, insert_bounds.top);
			}
			sum
		} else { unlinked_table };
		expanded_tables.insert(path.clone(), linked_table);
	}
	info!("LINK PATHS FULL: {:?}", &paths);
	info!("LINK PATHS SHORT: {:?}", &paths.iter().map(|it|(it.last_story_id(), it.len())).collect::<Vec<_>>());
	for path in &paths {
		let table = expanded_tables.get(path).expect("expanded table");
		info!("LINKED TABLE STORY: {:?}, nearest_z: {}, furthest_z: {}", path.last_story_id(), table.nearest_z(), table.furthest_z());
	}
	let linked = expanded_tables.get(&root_path).cloned().expect("root table");
	linked
}
