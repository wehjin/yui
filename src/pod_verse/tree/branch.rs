use crate::{Bounds, StoryId};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct PodBranch {
	pub story_id: StoryId,
	pub bounds: Bounds,
}
