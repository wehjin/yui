use std::sync::Arc;

use crate::{Bounds, DrawPad};
use crate::layout::LayoutContext;
use crate::story_id::StoryId;
use crate::yard::{ArcYard, Yard};

pub fn story(id: i32, story_id: StoryId) -> ArcYard {
	Arc::new(StoryYard { yard_id: id, story_id })
}

struct StoryYard {
	yard_id: i32,
	story_id: StoryId,
}

impl Yard for StoryYard {
	fn id(&self) -> i32 { self.yard_id }
	fn layout(&self, ctx: &mut LayoutContext) -> usize {
		let (bounds_id, _bounds) = ctx.edge_bounds();
		ctx.set_yard_bounds(self.yard_id, bounds_id);
		ctx.add_dependency(self.yard_id, self.story_id);
		bounds_id
	}
	fn render(&self, bounds: &Bounds, _focus_id: i32, pad: &mut dyn DrawPad) -> Option<Vec<(ArcYard, Option<i32>)>> {
		pad.story(bounds, self.story_id);
		None
	}
}

