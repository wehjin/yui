use std::cell::RefCell;

use crate::{Bounds, DrawPad, FillColor, FillGrade, StrokeColor};
use crate::spot::SpotFront;
use crate::yui_curses::spot::spot_stack::SpotStack;

pub struct SpotTable {
	rows: i32,
	cols: i32,
	spots: Vec<RefCell<SpotStack>>,
}


impl SpotTable {
	pub fn new(height: i32, width: i32) -> Self {
		let origin_stack = SpotStack::new();
		SpotTable {
			rows: height,
			cols: width,
			spots: vec![origin_stack; (width * height) as usize].into_iter().map(|it| RefCell::new(it)).collect(),
		}
	}
	pub fn width_height(&self) -> (i32, i32) { (self.cols, self.rows) }

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
}