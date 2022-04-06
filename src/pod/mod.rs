use crate::ArcYard;
use crate::spot::spot_table::SpotTable;

pub mod yard;

pub trait Pod {
	fn set_yard(&mut self, yard: ArcYard);
	fn set_size(&mut self, width_height: (i32, i32));
	fn focus_up(&mut self);
	fn focus_down(&mut self);
	fn focus_left(&mut self);
	fn focus_right(&mut self);
	fn insert_char(&self, char: char);
	fn insert_space(&self);
	fn layout_and_render(&mut self) -> &SpotTable;
}


