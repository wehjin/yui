use crate::{ArcYard, Before, Cling, Pack, Padding, SenderLink};
use crate::palette::FillGrade::Select;
use crate::palette::StrokeColor;
use crate::yard::Pressable;
use crate::yui::prelude::yard;

struct Table {
	id: i32,
	focus: usize,
	headers: Vec<(usize, String)>,
	rows: Vec<Vec<String>>,
	link: SenderLink<usize>,
}

const CELL_PADDING: i32 = 2;

impl Table {
	pub fn into_yard(self) -> ArcYard {
		let Table { id, focus, headers, rows, link } = self;
		let header_row = headers
			.iter()
			.rfold(yard::empty(), |row, (width, label)| {
				let label = yard::label(label.to_string(), StrokeColor::BodyOnBackground, Cling::LeftBottom);
				row.pack_left(*width as i32, label.pad_cols(CELL_PADDING))
			})
			;
		let items = rows.iter()
			.enumerate()
			.map(|(row_index, row_labels)| {
				let yard = row_labels.iter()
					.enumerate()
					.rfold(yard::empty(), |row, (i, label)| {
						let label = yard::label(label.to_string(), StrokeColor::BodyOnBackground, Cling::Left);
						let width = headers[i].0 as i32;
						row.pack_left(width, label.pad_cols(CELL_PADDING))
					});
				(3u8, yard.pressable(link.map(move |_| row_index)))
			})
			.collect();
		yard::list(id, focus, items)
			.pack_top(1, yard::glyph(StrokeColor::CommentOnBackground, || '_'))
			.pack_top(2, header_row)
			.before(yard::grade(Select))
	}
}

pub fn table(id: i32, focus: usize, headers: Vec<(usize, String)>, rows: Vec<Vec<String>>, link: SenderLink<usize>) -> ArcYard {
	Table { id, focus, headers, rows, link }.into_yard()
}

