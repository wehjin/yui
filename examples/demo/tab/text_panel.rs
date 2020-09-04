use yui::{AfterFlow, ArcYard, Cling, Create, Flow, Link, Padding, Spark, yard};
use yui::palette::StrokeColor;

#[derive(Debug)]
pub struct TextDemo {}

impl Spark for TextDemo {
	type State = ();
	type Action = ();
	type Report = usize;

	fn create(&self, _ctx: &Create<Self::Action, Self::Report>) -> Self::State { () }

	fn flow(&self, _action: Self::Action, _ctx: &impl Flow<Self::State, Self::Action, Self::Report>) -> AfterFlow<Self::State, Self::Report> {
		AfterFlow::Ignore
	}

	fn render(_state: &Self::State, _link: &Link<Self::Action>) -> Option<ArcYard> {
		let trellis = yard::trellis(1, 1, vec![
			yard::label("[left]", StrokeColor::BodyOnBackground, Cling::Left),
			yard::label("[日本語]", StrokeColor::BodyOnBackground, Cling::Center),
			yard::label("[right]", StrokeColor::BodyOnBackground, Cling::Right),
		]);
		let rendering = trellis.pad(2);
		Some(rendering)
	}
}