use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

use crate::{Bounds, DrawPad, FillColor, FillGrade, StoryId, StrokeColor};
use crate::spot::SpotFront;
use crate::yui_curses::spot::spot_stack::SpotStack;

#[derive(Debug, Clone)]
pub struct SpotTable {
	rows: i32,
	cols: i32,
	spots: Vec<RefCell<SpotStack>>,
	seams: HashMap<StoryId, HashSet<Bounds>>,
}

impl SpotTable {
	pub fn new(height: i32, width: i32) -> Self {
		let origin_stack = SpotStack::new();
		SpotTable {
			rows: height,
			cols: width,
			spots: vec![origin_stack; (width * height) as usize].into_iter().map(|it| RefCell::new(it)).collect(),
			seams: HashMap::new(),
		}
	}
	pub fn expand_seam(&self, z: i32, depth: i32, exclude: (&StoryId, &Bounds)) -> Self {
		let spots = self.spots.iter().map(|spot| {
			let spot = spot.borrow();
			let new_spot = spot.expand_seam(z, depth);
			RefCell::new(new_spot)
		}).collect::<Vec<_>>();
		let mut seams = self.seams.clone();
		{
			let keys = seams.keys().cloned().collect::<Vec<_>>();
			for story_id in keys {
				if let Some(set) = seams.remove(&story_id) {
					let new_set = set.into_iter().filter_map(|bounds| {
						if story_id == *exclude.0 && bounds == *exclude.1 {
							None
						} else {
							Some(bounds.expand_seam(z, depth))
						}
					}).collect::<HashSet<_>>();
					if new_set.len() > 0 {
						seams.insert(story_id, new_set);
					}
				}
			}
		}
		SpotTable { rows: self.rows, cols: self.cols, spots, seams }
	}
	pub fn insert_seam(&mut self, from: &SpotTable, z: i32, left: i32, top: i32) {
		for from_x in 0..from.cols {
			let to_x = from_x + left;
			if to_x >= 0 && to_x < self.cols {
				for from_y in 0..from.rows {
					let to_y = from_y + top;
					if to_y >= 0 && to_y < self.rows {
						let mut to_cell = self.spot_stack(to_y, to_x).borrow_mut();
						let from_cell = from.spot_stack(from_y, from_x).borrow();
						to_cell.insert_seam(z, from_cell.borrow());
					}
				}
			}
		}
		from.seams.iter().for_each(|(from_story_id, from_bounds)| {
			let mut hash_set = self.seams.remove(from_story_id).unwrap_or_else(|| HashSet::new());
			from_bounds.iter().for_each(|from_bounds| {
				let shifted_bounds = from_bounds.shift_seam(z, left, top);
				hash_set.insert(shifted_bounds);
			});
			self.seams.insert(*from_story_id, hash_set);
		});
	}

	pub fn width_height(&self) -> (i32, i32) { (self.cols, self.rows) }
	pub fn nearest_z(&self) -> i32 {
		self.spots.iter().fold(0, |result, next| {
			let stack = next.borrow();
			let nearest = stack.nearest_z(result);
			nearest
		})
	}
	pub fn furthest_z(&self) -> i32 {
		self.spots.iter().fold(i32::MIN, |result, next| {
			let stack = next.borrow();
			let nearest = stack.furthest_z(result);
			nearest
		})
	}
	pub fn to_seams(&self) -> Vec<(StoryId, Bounds)> {
		self.seams.clone().into_iter().map(|(story_id, bounds)| {
			bounds.into_iter().map(move |bound| (story_id, bound))
		}).flatten().collect::<Vec<_>>()
	}

	pub fn spot_stack(&self, y: i32, x: i32) -> &RefCell<SpotStack> {
		let index = y * self.cols + x;
		&self.spots[index as usize]
	}

	pub fn each<F: Fn(i32, i32, &SpotFront)>(&self, f: F) {
		for y in 0..self.rows {
			for x in 0..self.cols {
				let stack = self.spot_stack(y, x).borrow();
				let front = stack.to_front();
				f(y, x, &front);
			}
		}
	}

	pub fn to_fronts(&self) -> Vec<Vec<SpotFront>> {
		let mut table = Vec::new();
		for i in 0..self.spots.len() {
			if i as i32 % self.cols == 0 {
				table.insert(0, Vec::new());
			}
			let stack = &self.spots[i];
			let front = stack.borrow().to_front();
			table[0].push(front);
		}
		table.reverse();
		table
	}
}

impl DrawPad for SpotTable {
	fn fill(&mut self, bounds: &Bounds, color: FillColor) {
		for (x, y, z) in bounds.iter() {
			if y >= 0 && y < self.rows && x >= 0 && x < self.cols {
				let stack = self.spot_stack(y, x);
				stack.borrow_mut().set_fill(color, z);
			}
		}
	}

	fn grade(&mut self, bounds: &Bounds, grade: FillGrade) {
		for (x, y, z) in bounds.iter() {
			if y >= 0 && y < self.rows && x >= 0 && x < self.cols {
				let stack = self.spot_stack(y, x);
				stack.borrow_mut().set_fill_grade(grade, z);
			}
		}
	}

	fn glyph(&mut self, bounds: &Bounds, glyph: &str, color: StrokeColor) {
		for (x, y, z) in bounds.iter() {
			if y >= 0 && y < self.rows && x >= 0 && x < self.cols {
				let chars = glyph.chars().collect::<Vec<_>>();
				let string_index = x - bounds.left;
				if string_index >= 0 && string_index < (chars.len() as i32) {
					let start = string_index as usize;
					let sub_glyph: String = chars[start..(start + 1)].iter().collect();
					self.spot_stack(y, x).borrow_mut().set_stroke(sub_glyph, color, z)
				}
			}
		}
	}

	fn dark(&mut self, bounds: &Bounds, exclude: &Bounds) {
		for (x, y, z) in bounds.iter() {
			if y >= 0 && y < self.rows && x >= 0 && x < self.cols {
				if !exclude.intersects(y, x) {
					let stack = self.spot_stack(y, x);
					stack.borrow_mut().set_dark(z)
				}
			}
		}
	}

	fn story(&mut self, bounds: &Bounds, story_id: StoryId) {
		let mut story_bounds = self.seams.remove(&story_id).unwrap_or_else(|| HashSet::new());
		story_bounds.insert(bounds.clone());
		self.seams.insert(story_id, story_bounds);
	}
}