use std::cell::{Cell, RefCell};
use std::collections::HashMap;

use ncurses::{init_color, init_pair, start_color, use_default_colors};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum FillColor {
	Background,
	Primary,
}

impl From<FillColor> for i16 {
	fn from(color: FillColor) -> Self {
		match color {
			FillColor::Background => 0,
			FillColor::Primary => 1
		}
	}
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum StrokeColor {
	Body,
}

impl From<StrokeColor> for i16 {
	fn from(color: StrokeColor) -> Self {
		match color {
			StrokeColor::Body => 8
		}
	}
}

#[derive(Debug)]
pub struct Palette {
	indices: RefCell<HashMap<(StrokeColor, FillColor), i16>>,
	next_index: Cell<i16>,
}

impl Palette {
	pub fn new() -> Self {
		start_color();
		use_default_colors();
		init_color_by_parts(i16::from(FillColor::Background), BASE3);
		init_color_by_parts(i16::from(FillColor::Primary), BASE02);
		init_color_by_parts(i16::from(StrokeColor::Body), BASE00);
		Palette {
			indices: RefCell::new(HashMap::new()),
			next_index: Cell::new(1),
		}
	}

	pub fn color_pair_index(&self, stroke: StrokeColor, fill: FillColor) -> i16 {
		let stroke_fill = (stroke, fill);
		match self.existing_index(&stroke_fill) {
			None => {
				let index = self.advance_index();
				init_pair(index, i16::from(stroke), i16::from(fill));
				self.indices.borrow_mut().insert(stroke_fill, index);
				index
			}
			Some(index) => index
		}
	}

	fn existing_index(&self, stroke_fill: &(StrokeColor, FillColor)) -> Option<i16> {
		self.indices.borrow().get(&stroke_fill).map(|it| it.to_owned())
	}

	fn advance_index(&self) -> i16 {
		let index = self.next_index.get();
		self.next_index.set(index + 1);
		index
	}
}

const BASE02: [i32; 3] = [7, 54, 66];
const BASE00: [i32; 3] = [101, 123, 131];
const BASE3: [i32; 3] = [253, 246, 227];

fn init_color_by_parts(color: i16, parts: [i32; 3]) {
	init_color(
		color,
		to_component(parts[0]),
		to_component(parts[1]),
		to_component(parts[2]),
	);
}

fn to_component(color8: i32) -> i16 {
	(color8 as f32 / 256.0 * 1000.0) as i16
}

