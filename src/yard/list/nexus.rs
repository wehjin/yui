use crate::yard::list::nexus::Nexus::Down;
use crate::core::bounds::Bounds;

#[cfg(test)]
mod tests {
	use crate::yard::list::nexus::Nexus;

	#[test]
	fn small_list() {
		let item_heights = vec![2, 3];
		let item_tops = vec![0, 2];
		let list_height = item_heights.iter().fold(0, |sum, part| (sum + *part));
		let top_nexus = Nexus::new(0, &item_heights);
		let bottom_nexus = top_nexus.down(&item_heights).expect("top_nexus.down");
		let top_pivot_row = top_nexus.pivot_row(10, 2, list_height, 2, &item_tops);
		let bottom_pivot_row = bottom_nexus.pivot_row(10, 2, list_height, 2, &item_tops);
		assert_eq!(top_pivot_row, 2);
		assert_eq!(bottom_pivot_row, 6);
	}

	#[test]
	fn large_list() {
		let item_heights = vec![2, 3];
		let item_tops = vec![0, 2];
		let list_height = item_heights.iter().fold(0, |sum, part| (sum + *part));
		let top_nexus = Nexus::new(0, &item_heights);
		let bottom_nexus = top_nexus.down(&item_heights).expect("top_nexus.down");
		let top_pivot_row = top_nexus.pivot_row(4, 0, list_height, 2, &item_tops);
		let bottom_pivot_row = bottom_nexus.pivot_row(4, 0, list_height, 2, &item_tops);
		assert_eq!(top_pivot_row, 0);
		assert_eq!(bottom_pivot_row, 3);
	}

	#[test]
	fn one_heights() {
		let item_heights = vec![2, 3];
		let nexus = Nexus::new(0, &item_heights);
		let u = nexus.up(&item_heights);
		let d = nexus.down(&item_heights).expect("nexus down");
		let dd = d.down(&item_heights);
		let du = d.up(&item_heights).expect("up on down nexus");
		assert_eq!(u, None);
		assert_eq!(d, Nexus::Down { last_pos: 4, item_index: 1, max_index: 2 });
		assert_eq!(dd, None);
		assert_eq!(du, Nexus::Up { first_pos: 0, item_index: 0, max_index: 2 });
	}

	#[test]
	fn one_height() {
		let item_heights = vec![2];
		let nexus = Nexus::new(0, &item_heights);
		let d = nexus.down(&item_heights);
		assert_eq!(d, None);
		let u = nexus.up(&item_heights);
		assert_eq!(u, None);
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Nexus {
	Up { first_pos: i32, item_index: usize, max_index: usize },
	Down { last_pos: i32, item_index: usize, max_index: usize },
}

impl Nexus {
	pub fn can_up(&self) -> bool { self.item_index() > 0 }
	pub fn can_down(&self) -> bool { (self.item_index() + 1) < self.max_index() }
	pub fn down(&self, item_heights: &Vec<i32>) -> Option<Self> {
		match self {
			Nexus::Up { first_pos: first_row, item_index, max_index } => {
				let next_item_index = *item_index + 1;
				if next_item_index >= item_heights.len() {
					None
				} else {
					let next_item_height = item_heights[next_item_index];
					assert!(next_item_height > 0);
					let next_first_row = *first_row + item_heights[*item_index];
					let next_last_row = next_first_row + next_item_height - 1;
					Some(Down { last_pos: next_last_row, item_index: next_item_index, max_index: *max_index })
				}
			}
			Nexus::Down { last_pos: last_row, item_index, max_index } => {
				let next_item_index = *item_index + 1;
				if next_item_index >= item_heights.len() {
					None
				} else {
					let next_item_height = item_heights[next_item_index];
					assert!(next_item_height > 0);
					let next_last_row = *last_row + next_item_height;
					Some(Down { last_pos: next_last_row, item_index: next_item_index, max_index: *max_index })
				}
			}
		}
	}
	pub fn up(&self, item_heights: &Vec<i32>) -> Option<Self> {
		match self {
			Nexus::Up { first_pos: first_row, item_index, max_index } => {
				if *item_index == 0 {
					None
				} else {
					let next_item_index = *item_index - 1;
					let next_item_height = item_heights[next_item_index];
					assert!(next_item_height > 0);
					let next_first_row = *first_row - next_item_height;
					Some(Nexus::Up { first_pos: next_first_row, item_index: next_item_index, max_index: *max_index })
				}
			}
			Nexus::Down { last_pos: last_row, item_index, max_index } => {
				if *item_index == 0 {
					None
				} else {
					let next_index_index = *item_index - 1;
					let next_item_height = item_heights[next_index_index];
					assert!(next_item_height > 0);
					let item_height = item_heights[*item_index];
					let next_last_row = *last_row - item_height;
					let next_first_row = next_last_row - next_item_height + 1;
					Some(Nexus::Up { first_pos: next_first_row, item_index: next_index_index, max_index: *max_index })
				}
			}
		}
	}
	pub fn pivot_row(&self, bounds_height: i32, bounds_top: i32, list_height: i32, min_item_height: i32, item_tops: &Vec<i32>) -> i32 {
		if list_height <= bounds_height {
			bounds_top + match self {
				Nexus::Up { .. } => item_tops[self.item_index()],
				Nexus::Down { item_index, .. } => {
					if item_index + 1 == item_tops.len() {
						list_height - 1
					} else {
						item_tops[item_index + 1] - 1
					}
				}
			}
		} else {
			let pivot_factor = ((self.pivot_pos() + 1) as f64) / (list_height as f64);
			let float_rows_from_top = pivot_factor * bounds_height as f64;
			let rows_from_top = (float_rows_from_top - 0.000001).floor() as i32;
			let pivot_row_offset = {
				match self {
					Nexus::Up { item_index, max_index, .. } => {
						let index_extra = ((*max_index - *item_index - 1) as i32).min(bounds_height / 5);
						rows_from_top.min((bounds_height - min_item_height) - index_extra)
							.max(if *item_index == 0 { 0 } else { 1 })
					}
					Nexus::Down { item_index, max_index, .. } => {
						let index_extra = (*item_index as i32).min(bounds_height as i32 / 5);
						rows_from_top.max((min_item_height - 1) + index_extra)
							.min(bounds_height - 1 - if *item_index + 1 >= *max_index { 0 } else { 1 })
					}
				}
			};
			bounds_top + pivot_row_offset
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
	pub fn max_index(&self) -> usize {
		match self {
			Nexus::Up { max_index, .. } => *max_index,
			Nexus::Down { max_index, .. } => *max_index,
		}
	}
	pub fn new(index: usize, item_heights: &Vec<i32>) -> Self {
		let mut nexus = Nexus::Up { first_pos: 0, item_index: 0, max_index: item_heights.len() };
		for _ in 0..index {
			nexus = nexus.down(item_heights).expect("nexus down");
		}
		nexus
	}
}
