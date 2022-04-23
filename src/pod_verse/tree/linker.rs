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
	let mut expanded_focus_map: HashMap<PodPath, ActiveFocus> = HashMap::new();
	for parent_path in all_paths_longest_first {
		let path = &parent_path;
		let unlinked_focus = active_focus_at_path(&path, &expanded_focus_map, layout_map).cloned().unwrap_or_else(|| ActiveFocus::default());
		let linked_focus = if let Some(direct_children) = children.get(&parent_path) {
			let children_nearest_first = {
				let mut vec = direct_children.iter().collect::<Vec<_>>();
				vec.sort_by_key(|path| path.last_bounds().z);
				vec
			};
			children_nearest_first.into_iter().fold(unlinked_focus, |mut sum, child_path| {
				let insertion_bounds = child_path.last_bounds();
				let insertion_point = insertion_bounds.z;
				let child_focus = expanded_focus_map.get(child_path).cloned().unwrap_or_else(|| {
					warn!("Missing expanded focus at path {:?}", &child_path);
					ActiveFocus::default()
				});
				let child_depth = child_focus.nearest_z().abs() + 1;
				sum.expand_seam(insertion_point, child_depth);
				sum.insert_seam(&child_focus, insertion_point, insertion_bounds.left, insertion_bounds.top);
				sum
			})
		} else {
			unlinked_focus
		};
		expanded_focus_map.insert(parent_path, linked_focus);
	};
	expanded_focus_map
}

fn active_focus_at_path<'a>(path: &PodPath, expanded_focus_map: &'a HashMap<PodPath, ActiveFocus>, layout_map: &'a HashMap<PodPath, LayoutState>) -> Option<&'a ActiveFocus> {
	let expanded = expanded_focus_map.get(path);
	if expanded.is_some() {
		expanded
	} else {
		layout_map.get(&path).map(|it| &it.active_focus)
	}
}


pub fn link_spot_tables(spot_tables: &HashMap<PodPath, SpotTable>, children: &HashMap<PodPath, HashSet<PodPath>>, root_path: &PodPath) -> SpotTable {
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
			let fallback_table = SpotTable::new(0, 0);
			for child in children_nearest_first {
				let child_story = child.last_story_id();
				let insert_bounds = child.last_bounds();
				let child_table = expanded_tables.get(child).unwrap_or_else(|| {
					warn!("Missing expanded table at path {:?}", &child);
					&fallback_table
				});
				let child_depth = child_table.nearest_z().abs() + 1;
				let insertion_point = insert_bounds.z;
				sum = sum.expand_seam(insertion_point, child_depth, (child_story, insert_bounds));
				sum.insert_seam(child_table, insertion_point, insert_bounds.left, insert_bounds.top);
			}
			sum
		} else { unlinked_table };
		expanded_tables.insert(path.clone(), linked_table);
	}
	let linked = expanded_tables.get(&root_path).cloned().expect("root table");
	linked
}
