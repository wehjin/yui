use std::collections::{HashMap, HashSet};

use crate::pod_verse::tree::PodPath;
use crate::spot::spot_table::SpotTable;

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
