use crate::{ArcYard, Trigger};
use crate::spot::spot_table::SpotTable;

pub mod yard;
pub mod link_pod;

pub trait Pod {
	fn set_yard(&mut self, yard: ArcYard);
	fn set_width_height(&mut self, width_height: (i32, i32));
	fn focus_up(&mut self);
	fn focus_down(&mut self);
	fn focus_left(&mut self);
	fn focus_right(&mut self);
	fn insert_char(&self, char: char);
	fn insert_space(&self);
	fn set_refresh_trigger(&mut self, trigger: Trigger);
	fn spot_table(&self) -> Option<SpotTable>;
}


