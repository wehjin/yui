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
