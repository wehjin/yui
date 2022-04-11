use std::thread;
use std::time::Duration;

use crate::{AfterFlow, ArcYard, Cling, Create, Flow, SenderLink, Spark, StoryVerse, StrokeColor, yard};
use crate::app::Edge;
use crate::pod_verse::PodVerse;
use crate::story_id::StoryId;

#[test]
fn simple_sub_story() {
	let main_id = StoryId::new(0);
	let (story_verse, _main_link) = StoryVerse::build(TwoWords { left: "Hello".into(), right: "World".into() }, main_id);
	thread::sleep(Duration::from_millis(1));
	assert_eq!(story_verse.read_stats().story_count, 3);
	let pod_verse = PodVerse::build(&story_verse, main_id);
	thread::sleep(Duration::from_millis(1));
	assert_eq!(pod_verse.read_pod_count(), 3);
}

struct Word {
	chars: String,
}

impl Spark for Word {
	type State = String;
	type Action = ();
	type Report = ();
	fn create<E: Edge + Clone + Send + 'static>(&self, _ctx: &Create<Self::Action, Self::Report, E>) -> Self::State { self.chars.to_string() }
	fn flow(&self, _action: Self::Action, _ctx: &impl Flow<Self::State, Self::Action, Self::Report>) -> AfterFlow<Self::State, Self::Report> { AfterFlow::Ignore }
	fn render(state: &Self::State, _link: &SenderLink<Self::Action>) -> Option<ArcYard> {
		Some(yard::label(state.clone(), StrokeColor::BodyOnBackground, Cling::Left))
	}
}

struct TwoWords {
	left: String,
	right: String,
}

impl Spark for TwoWords {
	type State = (StoryId, StoryId);
	type Action = ();
	type Report = ();

	fn create<E: Edge + Clone + Send + 'static>(&self, ctx: &Create<Self::Action, Self::Report, E>) -> Self::State {
		if let Some(edge) = ctx.edge() {
			(edge.sub_story(Word { chars: self.left.clone() }, None).story_id,
			 edge.sub_story(Word { chars: self.right.clone() }, None).story_id)
		} else {
			(StoryId::random(), StoryId::random())
		}
	}

	fn flow(&self, _action: Self::Action, _ctx: &impl Flow<Self::State, Self::Action, Self::Report>) -> AfterFlow<Self::State, Self::Report> { AfterFlow::Ignore }
	fn render(_state: &Self::State, _link: &SenderLink<Self::Action>) -> Option<ArcYard> { None }
}
