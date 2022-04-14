use crate::{Bounds, StoryId};
use crate::pod_verse::tree::branch::PodBranch;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct PodPath(Vec<PodBranch>);

impl PodPath {
	pub fn new(story_id: StoryId, bounds: Bounds) -> Self {
		PodPath(vec![PodBranch { story_id, bounds }])
	}
	pub fn last_branch(&self) -> &PodBranch { self.0.last().unwrap() }
	pub fn append(&self, story_id: StoryId, bounds: Bounds) -> Self {
		self.append_branch(PodBranch { story_id, bounds })
	}
	pub fn append_branch(&self, branch: PodBranch) -> Self {
		let mut vec = self.0.clone();
		vec.push(branch);
		PodPath(vec)
	}
	pub fn len(&self) -> usize { self.0.len() }
	pub fn last_story_id(&self) -> &StoryId { &self.last_branch().story_id }
	pub fn last_bounds(&self) -> &Bounds { &self.last_branch().bounds }
}


#[cfg(test)]
mod tests {
	use crate::{Bounds, StoryId};
	use crate::pod_verse::tree::path::PodPath;

	#[test]
	fn path_extension() {
		let story_ids = vec![StoryId::new(1), StoryId::new(2), StoryId::new(3)];
		let bounds = vec![Bounds::new(10, 10), Bounds::new(3, 3).with_z(3), Bounds::new(2, 2).with_z(7)];
		let path = PodPath::new(story_ids[0], bounds[0])
			.append(story_ids[1], bounds[1])
			.append(story_ids[2], bounds[2]);
		assert_eq!(3, path.len());
	}
}
