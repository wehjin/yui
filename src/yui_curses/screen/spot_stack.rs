use crate::palette::{FillColor, FillGrade, Palette, StrokeColor};

#[derive(Copy, Clone, Debug)]
pub(crate) struct SpotStack<'a> {
	fill_color: FillColor,
	fill_grade: FillGrade,
	fill_z: i32,
	stroke_type: Option<(char, StrokeColor)>,
	stroke_z: i32,
	dark_z: i32,
	palette: &'a Palette,
}

impl<'a> SpotStack<'a> {
	pub fn new(palette: &'a Palette) -> Self {
		SpotStack {
			fill_color: FillColor::Background,
			fill_grade: FillGrade::Plain,
			fill_z: i32::max_value(),
			stroke_type: Option::None,
			stroke_z: i32::max_value(),
			dark_z: i32::max_value(),
			palette,
		}
	}

	pub fn set_dark(&mut self, z: i32) {
		if z <= self.dark_z {
			self.dark_z = z;
		}
	}

	pub fn set_fill(&mut self, color: FillColor, z: i32) {
		if z <= self.fill_z {
			self.fill_z = z;
			self.fill_color = color;
		}
	}

	pub fn set_stroke(&mut self, glyph: char, color: StrokeColor, z: i32) {
		if z <= self.stroke_z {
			self.stroke_z = z;
			self.stroke_type = Option::Some((glyph, color))
		}
	}

	pub fn color_details(&self) -> (i16, char, bool) {
		let (glyph, stroke_color) = match self.stroke_type {
			None => (' ', StrokeColor::BodyOnBackground),
			Some((glyph, color)) =>
				if self.stroke_z <= self.fill_z {
					(glyph, color)
				} else {
					(' ', StrokeColor::BodyOnBackground)
				},
		};
		let darken = self.dark_z < self.fill_z;
		let color_pair = self.palette.color_pair_index(stroke_color, self.fill_color, self.fill_grade, darken);
		(color_pair, glyph, darken)
	}
}
