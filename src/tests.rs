use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

use rand::random;

use crate::{AfterFlow, ArcYard, Cling, Create, FillColor, FillGrade, Flow, Pack, Sendable, SenderLink, Spark, StoryVerse, StrokeColor, yard};

use crate::pod::Pod;
use crate::pod_verse::PodVerse;
use crate::story_id::StoryId;
use crate::super_story::SuperStory;

#[derive(Clone)]
enum TestAction {
	Refresh
}

impl Sendable for TestAction {}

#[test]
fn sub_story_rendering() {
	struct Main {}
	impl Spark for Main {
		type State = StoryId;
		type Action = ();
		type Report = ();
		fn create(&self, ctx: &Create<Self::Action, Self::Report>) -> Self::State {
			if let Some(edge) = ctx.edge() {
				edge.sub_story(PlainPrimary {}, None).story_id
			} else {
				StoryId::random()
			}
		}
		fn flow(&self, _action: Self::Action, _ctx: &impl Flow<Self::State, Self::Action, Self::Report>) -> AfterFlow<Self::State, Self::Report> { AfterFlow::Ignore }
		fn render(state: &Self::State, _link: &SenderLink<Self::Action>) -> Option<ArcYard> {
			let yard = yard::empty().pack_right(1, yard::story(random(), state.clone()));
			Some(yard)
		}
	}
	struct PlainPrimary {}
	impl Spark for PlainPrimary {
		type State = ();
		type Action = ();
		type Report = ();
		fn create(&self, _ctx: &Create<Self::Action, Self::Report>) -> Self::State { () }
		fn flow(&self, _action: Self::Action, _ctx: &impl Flow<Self::State, Self::Action, Self::Report>) -> AfterFlow<Self::State, Self::Report> { AfterFlow::Ignore }
		fn render(_state: &Self::State, _link: &SenderLink<Self::Action>) -> Option<ArcYard> {
			Some(yard::fill(FillColor::Primary, FillGrade::Plain))
		}
	}
	let main_id = StoryId::new(0);
	let (story_verse, _main_link) = StoryVerse::build(Main {}, main_id);
	thread::sleep(Duration::from_millis(1));
	let pod_verse = PodVerse::build(&story_verse);
	thread::sleep(Duration::from_millis(1));
	let mut main_pod = pod_verse.to_main_pod(SenderLink::ignore());
	main_pod.set_width_height((2, 1));
	let (action_link, action_source) = channel();
	main_pod.set_refresh_trigger(TestAction::Refresh.into_trigger(&action_link));
	let _action = action_source.recv().expect("test action");
	let spot_table = main_pod.spot_table().unwrap();
	let fronts = spot_table.to_fronts().iter().map(|row| {
		row.iter().map(|spot| spot.fill_color).collect::<Vec<_>>()
	}).collect::<Vec<_>>();
	assert_eq!(fronts[0], vec![FillColor::Background, FillColor::Primary]);
}

#[test]
fn simple_sub_story() {
	let main_id = StoryId::new(0);
	let (story_verse, _main_link) = StoryVerse::build(TwoWords { left: "Hello".into(), right: "World".into() }, main_id);
	thread::sleep(Duration::from_millis(1));
	assert_eq!(story_verse.read_stats().story_count, 3);
	let pod_verse = PodVerse::build(&story_verse);
	thread::sleep(Duration::from_millis(1));
	let mut main_pod = pod_verse.to_main_pod(SenderLink::ignore());
	let (test_link, test_source) = channel();
	main_pod.set_refresh_trigger(TestAction::Refresh.into_trigger(&test_link));
	main_pod.set_width_height((3, 1));
	let mut success = false;
	let mut count = 0;
	for _i in [0..10] {
		let action = test_source.recv().unwrap();
		match action {
			TestAction::Refresh => {
				count += 1;
				if pod_verse.read_pod_count() >= 3 {
					success = true;
					break;
				}
			}
		}
		//let spot_table = main_pod.layout_and_render();
	}
	println!("REFRESH COUNT: {}", count);
	assert!(success);
}

struct Word {
	chars: String,
}

impl Spark for Word {
	type State = String;
	type Action = ();
	type Report = ();
	fn create(&self, _ctx: &Create<Self::Action, Self::Report>) -> Self::State { self.chars.to_string() }
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

	fn create(&self, ctx: &Create<Self::Action, Self::Report>) -> Self::State {
		if let Some(edge) = ctx.edge() {
			(edge.sub_story(Word { chars: self.left.clone() }, None).story_id,
			 edge.sub_story(Word { chars: self.right.clone() }, None).story_id)
		} else {
			(StoryId::random(), StoryId::random())
		}
	}

	fn flow(&self, _action: Self::Action, _ctx: &impl Flow<Self::State, Self::Action, Self::Report>) -> AfterFlow<Self::State, Self::Report> { AfterFlow::Ignore }
	fn render(state: &Self::State, _link: &SenderLink<Self::Action>) -> Option<ArcYard> {
		let yard = yard::fill(FillColor::Background, FillGrade::Plain)
			.pack_right(1, yard::story(random(), state.1))
			.pack_left(1, yard::story(random(), state.0));
		Some(yard)
	}
}
