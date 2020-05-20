use crate::yard::list::nexus::Nexus::Down;
use crate::yui::bounds::Bounds;

#[cfg(test)]
mod tests {
	use crate::yard::list::nexus::Nexus;

	#[test]
	fn small_list() {
		let item_heights = vec![2, 3];
		let list_height = item_heights.iter().fold(0, |sum, part| (sum + *part));
		let top_nexus = Nexus::new();
		let bottom_nexus = top_nexus.down(&item_heights).unwrap();
		let top_pivot_row = top_nexus.pivot_row(10, 2, list_height);
		let bottom_pivot_row = bottom_nexus.pivot_row(10, 2, list_height);
		assert_eq!(top_pivot_row, 2);
		assert_eq!(bottom_pivot_row, 2);
	}

	#[test]
	fn large_list() {
		let item_heights = vec![2, 3];
		let list_height = item_heights.iter().fold(0, |sum, part| (sum + *part));
		let top_nexus = Nexus::new();
		let bottom_nexus = top_nexus.down(&item_heights).unwrap();
		let top_pivot_row = top_nexus.pivot_row(4, 0, list_height);
		let bottom_pivot_row = bottom_nexus.pivot_row(4, 0, list_height);
		assert_eq!(top_pivot_row, 0);
		assert_eq!(bottom_pivot_row, 3);
	}

	#[test]
	fn one_heights() {
		let item_heights = vec![2, 3];
		let nexus = Nexus::new();
		let u = nexus.up(&item_heights);
		let d = nexus.down(&item_heights).unwrap();
		let dd = d.down(&item_heights);
		let du = d.up(&item_heights).unwrap();
		assert_eq!(u, None);
		assert_eq!(d, Nexus::Down { last_pos: 4, item_index: 1 });
		assert_eq!(dd, None);
		assert_eq!(du, Nexus::Up { first_pos: 0, item_index: 0 });
	}

	#[test]
	fn one_height() {
		let item_heights = vec![2];
		let nexus = Nexus::new();
		let d = nexus.down(&item_heights);
		assert_eq!(d, None);
		let u = nexus.up(&item_heights);
		assert_eq!(u, None);
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Nexus {
	Up { first_pos: i32, item_index: usize },
	Down { last_pos: i32, item_index: usize },
}

impl Nexus {
	pub fn down(&self, item_heights: &Vec<i32>) -> Option<Self> {
		match self {
			Nexus::Up { first_pos: first_row, item_index } => {
				let next_item_index = *item_index + 1;
				if next_item_index >= item_heights.len() {
					None
				} else {
					let next_item_height = item_heights[next_item_index];
					assert!(next_item_height > 0);
					let next_first_row = *first_row + item_heights[*item_index];
					let next_last_row = next_first_row + next_item_height - 1;
					Some(Down { last_pos: next_last_row, item_index: next_item_index })
				}
			}
			Nexus::Down { last_pos: last_row, item_index } => {
				let next_item_index = *item_index + 1;
				if next_item_index >= item_heights.len() {
					None
				} else {
					let next_item_height = item_heights[next_item_index];
					assert!(next_item_height > 0);
					let next_last_row = *last_row + next_item_height;
					Some(Down { last_pos: next_last_row, item_index: next_item_index })
				}
			}
		}
	}
	pub fn up(&self, item_heights: &Vec<i32>) -> Option<Self> {
		match self {
			Nexus::Up { first_pos: first_row, item_index } => {
				if *item_index == 0 {
					None
				} else {
					let next_item_index = *item_index - 1;
					let next_item_height = item_heights[next_item_index];
					assert!(next_item_height > 0);
					let next_first_row = *first_row - next_item_height;
					Some(Nexus::Up { first_pos: next_first_row, item_index: next_item_index })
				}
			}
			Nexus::Down { last_pos: last_row, item_index } => {
				if *item_index == 0 {
					None
				} else {
					let next_index_index = *item_index - 1;
					let next_item_height = item_heights[next_index_index];
					assert!(next_item_height > 0);
					let item_height = item_heights[*item_index];
					let next_last_row = *last_row - item_height;
					let next_first_row = next_last_row - next_item_height + 1;
					Some(Nexus::Up { first_pos: next_first_row, item_index: next_index_index })
				}
			}
		}
	}
	pub fn pivot_row(&self, bounds_height: i32, bounds_top: i32, list_height: i32) -> i32 {
		if list_height <= bounds_height {
			bounds_top
		} else {
			let pivot_factor = ((self.pivot_pos() + 1) as f64) / (list_height as f64);
			let pivot_height = pivot_factor * bounds_height as f64;
			let pivot_row_offset = (pivot_height - 0.000001).floor() as i32;
			let pivot_row = bounds_top + pivot_row_offset;
			pivot_row
		}
	}
	pub fn item_bounds(&self, index: usize, bounds: &Bounds, pivot_row: i32, pivot_pos: i32, item_tops: &Vec<i32>, item_heights: &Vec<i32>) -> Bounds {
		let top_pos = item_tops[index];
		let height = item_heights[index];
		match self {
			Nexus::Up { .. } => {
				let top_pos_from_pivot = top_pos - pivot_pos;
				let top_row = pivot_row + top_pos_from_pivot;
				bounds.set_height_from_above(top_row - bounds.top, height)
			}
			Nexus::Down { .. } => {
				let bottom_pos = top_pos + height;
				let bottom_pos_from_pivot = bottom_pos - pivot_pos;
				let bottom_row = pivot_row + bottom_pos_from_pivot;
				bounds.set_height_from_below(bounds.bottom - bottom_row, height)
			}
		}
	}
	pub fn pivot_pos(&self) -> i32 {
		match self {
			Nexus::Up { first_pos, .. } => *first_pos,
			Nexus::Down { last_pos, .. } => *last_pos,
		}
	}
	pub fn item_index(&self) -> usize {
		match self {
			Nexus::Up { item_index, .. } => *item_index,
			Nexus::Down { item_index, .. } => *item_index,
		}
	}
	pub fn new() -> Self {
		Nexus::Up { first_pos: 0, item_index: 0 }
	}
}
