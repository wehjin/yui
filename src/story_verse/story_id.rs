use rand::random;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct StoryId(usize);

impl StoryId {
	pub fn new(id: usize) -> Self { StoryId(id) }
	pub fn random() -> Self { StoryId(random::<usize>()) }
	pub fn sub_id(&self) -> Self { Self::random() }
	pub fn dialog_id(&self) -> Self { Self::random() }
}
