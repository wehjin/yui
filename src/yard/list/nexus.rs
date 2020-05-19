use crate::yard::list::nexus::Nexus::Down;

#[cfg(test)]
mod tests {
	use crate::yard::list::nexus::Nexus;

	#[test]
	fn one_heights() {
		let item_heights = vec![2, 3];
		let nexus = Nexus::new();
		let u = nexus.up(&item_heights);
		let d = nexus.down(&item_heights).unwrap();
		let dd = d.down(&item_heights);
		let du = d.up(&item_heights).unwrap();
		assert_eq!(u, None);
		assert_eq!(d, Nexus::Down { last_row: 4, item_index: 1 });
		assert_eq!(dd, None);
		assert_eq!(du, Nexus::Up { first_row: 0, item_index: 0 });
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
	Up { first_row: i32, item_index: usize },
	Down { last_row: i32, item_index: usize },
}

impl Nexus {
	pub fn down(&self, item_heights: &Vec<i32>) -> Option<Self> {
		match self {
			Nexus::Up { first_row, item_index } => {
				let next_item_index = *item_index + 1;
				if next_item_index >= item_heights.len() {
					None
				} else {
					let next_item_height = item_heights[next_item_index];
					assert!(next_item_height > 0);
					let next_first_row = *first_row + item_heights[*item_index];
					let next_last_row = next_first_row + next_item_height - 1;
					Some(Down { last_row: next_last_row, item_index: next_item_index })
				}
			}
			Nexus::Down { last_row, item_index } => {
				let next_item_index = *item_index + 1;
				if next_item_index >= item_heights.len() {
					None
				} else {
					let next_item_height = item_heights[next_item_index];
					assert!(next_item_height > 0);
					let next_last_row = *last_row + next_item_height;
					Some(Down { last_row: next_last_row, item_index: next_item_index })
				}
			}
		}
	}
	pub fn up(&self, item_heights: &Vec<i32>) -> Option<Self> {
		match self {
			Nexus::Up { first_row, item_index } => {
				if *item_index == 0 {
					None
				} else {
					let next_item_index = *item_index - 1;
					let next_item_height = item_heights[next_item_index];
					assert!(next_item_height > 0);
					let next_first_row = *first_row - next_item_height;
					Some(Nexus::Up { first_row: next_first_row, item_index: next_item_index })
				}
			}
			Nexus::Down { last_row, item_index } => {
				if *item_index == 0 {
					None
				} else {
					let next_index_index = *item_index - 1;
					let next_item_height = item_heights[next_index_index];
					assert!(next_item_height > 0);
					let item_height = item_heights[*item_index];
					let next_last_row = *last_row - item_height;
					let next_first_row = next_last_row - next_item_height + 1;
					Some(Nexus::Up { first_row: next_first_row, item_index: next_index_index })
				}
			}
		}
	}
	pub fn item_index(&self) -> usize {
		match self {
			Nexus::Up { item_index, .. } => *item_index,
			Nexus::Down { item_index, .. } => *item_index,
		}
	}
	pub fn new() -> Self {
		Nexus::Up { first_row: 0, item_index: 0 }
	}
}
