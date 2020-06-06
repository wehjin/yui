use std::sync::Arc;

use crate::ArcYard;
use crate::yard::YardPublisher;

#[derive(Clone)]
pub struct State {
	pub era: usize,
	pub yard: ArcYard,
	pub back_to_front: Vec<Arc<dyn YardPublisher>>,
}

impl State {
	pub fn pop_front(&self) -> Self {
		let mut back_to_front = self.back_to_front.to_vec();
		back_to_front.pop();
		State {
			era: self.era + 1,
			yard: self.yard.to_owned(),
			back_to_front,
		}
	}
	pub fn push_front(&self, front: Arc<dyn YardPublisher>) -> Self {
		let mut back_to_front = self.back_to_front.to_vec();
		back_to_front.push(front);
		State {
			era: self.era + 1,
			yard: self.yard.to_owned(),
			back_to_front,
		}
	}
	pub fn set_yard(&self, yard: ArcYard) -> Self {
		State {
			era: self.era,
			yard,
			back_to_front: self.back_to_front.to_vec(),
		}
	}
}
