use crate::{FillColor, FillGrade, StrokeColor};

pub mod spot_stack;
pub mod spot_table;
pub mod spot_field;

#[derive(Clone, Debug)]
pub struct SpotFront {
	pub fill_color: FillColor,
	pub fill_grade: FillGrade,
	pub stroke: Option<(String, StrokeColor)>,
	pub dark: bool,
}
