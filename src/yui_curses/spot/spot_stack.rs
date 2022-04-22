use crate::palette::{FillColor, FillGrade, StrokeColor};
use crate::yui_curses::spot::spot_field::SpotField;
use crate::yui_curses::spot::SpotFront;

#[derive(Clone, Debug)]
pub struct SpotStack {
	fill_color: SpotField<FillColor>,
	fill_grade: SpotField<FillGrade>,
	stroke: SpotField<Option<(String, StrokeColor)>>,
	dark: SpotField<bool>,
}

impl SpotStack {
	pub fn new() -> Self {
		SpotStack {
			fill_color: SpotField::new(FillColor::Background),
			fill_grade: SpotField::new(FillGrade::Plain),
			stroke: SpotField::new(None),
			dark: SpotField::new(false),
		}
	}
	pub fn expand_seam(&self, z: i32, depth: i32) -> Self {
		SpotStack {
			fill_color: self.fill_color.expand_seam(z, depth),
			fill_grade: self.fill_grade.expand_seam(z, depth),
			stroke: self.stroke.expand_seam(z, depth),
			dark: self.dark.expand_seam(z, depth),
		}
	}
	pub fn insert_seam(&mut self, z: i32, stack: &SpotStack) {
		self.fill_color.insert_seam(z, &stack.fill_color);
		self.fill_grade.insert_seam(z, &stack.fill_grade);
		self.stroke.insert_seam(z, &stack.stroke);
		self.dark.insert_seam(z, &stack.dark);
	}
	pub fn nearest_z(&self, z: i32) -> i32 {
		let result = z;
		let result = self.fill_color.nearest_z(result);
		let result = self.fill_grade.nearest_z(result);
		let result = self.stroke.nearest_z(result);
		let result = self.dark.nearest_z(result);
		result
	}
	pub fn furthest_z(&self, z: i32) -> i32 {
		let result = z;
		let result = self.fill_color.furthest_z(result);
		let result = self.fill_grade.furthest_z(result);
		let result = self.stroke.furthest_z(result);
		let result = self.dark.furthest_z(result);
		result
	}
	pub fn set_fill(&mut self, color: FillColor, z: i32) {
		self.fill_color.set_near_equal(color, z);
	}

	pub fn set_fill_grade(&mut self, grade: FillGrade, z: i32) {
		self.fill_grade.set_near_equal(grade, z);
	}

	pub fn set_stroke(&mut self, glyph: String, color: StrokeColor, z: i32) {
		self.stroke.set_near_equal(Some((glyph, color)), z);
	}

	pub fn set_dark(&mut self, z: i32) {
		self.dark.set_near_equal(true, z);
	}

	pub fn to_front(&self) -> SpotFront {
		SpotFront {
			fill_color: self.fill_color.value,
			fill_grade: self.real_grade(),
			stroke: self.real_stroke(),
			dark: self.real_dark(),
		}
	}

	fn real_stroke(&self) -> Option<(String, StrokeColor)> {
		if self.stroke.z <= self.fill_color.z {
			self.stroke.value.clone()
		} else {
			None
		}
	}

	fn real_dark(&self) -> bool {
		self.dark.value && (self.dark.z <= self.fill_color.z)
	}

	fn real_grade(&self) -> FillGrade {
		if self.fill_grade.z <= self.fill_color.z {
			self.fill_grade.value
		} else {
			FillGrade::Plain
		}
	}
}
