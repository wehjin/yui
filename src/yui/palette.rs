use std::cell::{Cell, RefCell};
use std::collections::HashMap;

use ncurses::{init_color, init_pair, start_color, use_default_colors};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum FillColor {
	Background,
	Primary,
	BackgroundWithFocus,
}

impl From<FillColor> for i16 {
	fn from(color: FillColor) -> Self {
		match color {
			FillColor::Background => COLOR_BASE3,
			FillColor::Primary => COLOR_BASE02,
			FillColor::BackgroundWithFocus => COLOR_BASE1,
		}
	}
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum StrokeColor {
	Body,
	PrimaryBody,
	Comment,
	EnabledOnBackground,
}

impl From<StrokeColor> for i16 {
	fn from(color: StrokeColor) -> Self {
		match color {
			StrokeColor::Body => COLOR_BASE00,
			StrokeColor::PrimaryBody => COLOR_BASE1,
			StrokeColor::Comment => COLOR_BASE1,
			StrokeColor::EnabledOnBackground => COLOR_BASE02,
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
		init_color_by_parts(COLOR_BASE3, BASE3);
		init_color_by_parts(COLOR_BASE2, BASE2);
		init_color_by_parts(COLOR_BASE1, BASE1);
		init_color_by_parts(COLOR_BASE0, BASE0);
		init_color_by_parts(COLOR_BASE00, BASE00);
		init_color_by_parts(COLOR_BASE01, BASE01);
		init_color_by_parts(COLOR_BASE02, BASE02);
		init_color_by_parts(COLOR_BASE03, BASE03);
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

const COLOR_BASE03: i16 = 0;
const COLOR_BASE02: i16 = 1;
const COLOR_BASE01: i16 = 2;
const COLOR_BASE00: i16 = 3;
const COLOR_BASE0: i16 = 4;
const COLOR_BASE1: i16 = 5;
const COLOR_BASE2: i16 = 6;
const COLOR_BASE3: i16 = 7;

const BASE03: [i32; 3] = [0, 43, 54];
const BASE02: [i32; 3] = [7, 54, 66];
const BASE01: [i32; 3] = [88, 110, 117];
const BASE00: [i32; 3] = [101, 123, 131];
const BASE0: [i32; 3] = [131, 148, 150];
const BASE1: [i32; 3] = [147, 161, 161];
const BASE2: [i32; 3] = [238, 232, 213];
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

