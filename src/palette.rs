use std::cell::{Cell, RefCell};
use std::collections::HashMap;

use ncurses::{A_DIM, attr_t, COLOR_PAIR, init_color, init_pair, start_color, use_default_colors};

use crate::spot::SpotFront;

pub fn body_and_comment_for_fill(color: FillColor) -> (StrokeColor, StrokeColor) {
	match color {
		FillColor::Primary => (StrokeColor::BodyOnPrimary, StrokeColor::CommentOnPrimary),
		FillColor::Side => (StrokeColor::BodyOnSide, StrokeColor::CommentOnSide),
		FillColor::Background => (StrokeColor::BodyOnBackground, StrokeColor::CommentOnBackground),
	}
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum FillColor {
	Background,
	Primary,
	Side,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum FillGrade {
	Plain,
	Select,
	Focus,
	Press,
}

fn fill_i16(color: FillColor, grade: FillGrade, dimmed: bool) -> i16 {
	let flat = match color {
		FillColor::Primary => COLOR_BASE02,
		FillColor::Side => COLOR_BASE2,
		FillColor::Background => COLOR_BASE3,
	};
	let graded = match color {
		FillColor::Background | FillColor::Primary => match grade {
			FillGrade::Plain => flat,
			FillGrade::Select => center(1, flat),
			FillGrade::Focus => center(2, flat),
			FillGrade::Press => center(4, flat),
		}
		FillColor::Side => match grade {
			FillGrade::Select => center(-1, flat),
			FillGrade::Plain => flat,
			FillGrade::Focus => center(1, flat),
			FillGrade::Press => center(2, flat),
		}
	};
	if dimmed { darken(graded) } else { graded }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum StrokeColor {
	CommentOnBackground,
	CommentOnSide,
	BodyOnBackground,
	BodyOnSide,
	EnabledOnBackground,
	EnabledOnPrimary,
	BodyOnPrimary,
	CommentOnPrimary,
}

fn stroke_i16((color, dimmed): (StrokeColor, bool)) -> i16 {
	let color = match color {
		StrokeColor::CommentOnBackground => COLOR_BASE1,
		StrokeColor::CommentOnSide => COLOR_BASE00,
		StrokeColor::BodyOnBackground => COLOR_BASE00,
		StrokeColor::BodyOnSide => COLOR_BASE02,
		StrokeColor::EnabledOnBackground => COLOR_MAGENTA,
		StrokeColor::EnabledOnPrimary => COLOR_MAGENTA,
		StrokeColor::BodyOnPrimary => COLOR_BASE1,
		StrokeColor::CommentOnPrimary => COLOR_BASE00,
	};
	if dimmed { darken(color) } else { color }
}


#[derive(Debug)]
pub struct Palette {
	indices: RefCell<HashMap<(StrokeColor, FillColor, FillGrade, bool), i16>>,
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
		init_color_by_parts(COLOR_YELLOW, YELLOW);
		init_color_by_parts(COLOR_MAGENTA, MAGENTA);
		init_color_by_parts(COLOR_VIOLET, VIOLET);
		init_color_by_parts(COLOR_GREEN, GREEN);
		Palette {
			indices: RefCell::new(HashMap::new()),
			next_index: Cell::new(1),
		}
	}

	pub fn color_pair_index(&self, key: (StrokeColor, FillColor, FillGrade, bool)) -> i16 {
		match self.existing_index(&key) {
			Some(index) => index,
			None => {
				let (stroke, fill, fill_grade, dimmed) = key;
				let index = self.advance_index();
				init_pair(
					index,
					stroke_i16((stroke, dimmed)),
					fill_i16(fill, fill_grade, dimmed),
				);
				self.indices.borrow_mut().insert(key, index);
				index
			}
		}
	}

	fn existing_index(&self, index_key: &(StrokeColor, FillColor, FillGrade, bool)) -> Option<i16> {
		self.indices.borrow().get(&index_key).map(|it| it.to_owned())
	}

	fn advance_index(&self) -> i16 {
		let index = self.next_index.get();
		self.next_index.set(index + 1);
		index
	}

	fn to_cpi_glyph_dim<'a>(&self, front: &'a SpotFront) -> (i16, &'a str, bool) {
		let (glyph, stroke_color) = match front.stroke {
			None => (" ", StrokeColor::BodyOnBackground),
			Some((ref glyph, color)) => (glyph.as_str(), color),
		};
		let color_pair = self.color_pair_index((stroke_color, front.fill_color, front.fill_grade, front.dark));
		(color_pair, glyph, front.dark)
	}

	pub fn to_glyph_attr<'a>(&self, front: &'a SpotFront) -> Option<(&'a str, attr_t)> {
		let (color_pair_index, glyph, darken) = self.to_cpi_glyph_dim(front);
		if !glyph.is_empty() {
			let color_attr = COLOR_PAIR(color_pair_index);
			let attr = if darken { color_attr | A_DIM() } else { color_attr };
			Some((glyph, attr))
		} else {
			None
		}
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
const COLOR_YELLOW: i16 = 8;
const COLOR_MAGENTA: i16 = 9;
const COLOR_VIOLET: i16 = 10;
const COLOR_GREEN: i16 = 11;

fn darken(color_i16: i16) -> i16 {
	let delta = 3;
	if color_i16 < delta {
		0
	} else if color_i16 <= COLOR_BASE3 {
		color_i16 - delta
	} else {
		color_i16
	}
}

fn center(magnitude: i16, color_i16: i16) -> i16 {
	if color_i16 <= COLOR_BASE00 {
		(color_i16 + magnitude).min(COLOR_BASE00)
	} else if color_i16 <= COLOR_BASE3 {
		(color_i16 - magnitude).max(COLOR_BASE0)
	} else {
		color_i16
	}
}

const BASE03: [i32; 3] = [0, 43, 54];
const BASE02: [i32; 3] = [7, 54, 66];
const BASE01: [i32; 3] = [88, 110, 117];
const BASE00: [i32; 3] = [101, 123, 131];
const BASE0: [i32; 3] = [131, 148, 150];
const BASE1: [i32; 3] = [147, 161, 161];
const BASE2: [i32; 3] = [238, 232, 213];
const BASE3: [i32; 3] = [253, 246, 227];
const YELLOW: [i32; 3] = [181, 137, 0];
const MAGENTA: [i32; 3] = [211, 54, 130];
const VIOLET: [i32; 3] = [108, 113, 196];
const GREEN: [i32; 3] = [133, 153, 0];


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

