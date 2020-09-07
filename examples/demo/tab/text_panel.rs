use yui::palette::StrokeColor;
use yui::prelude::*;

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
		let light_innards = vec![
			yard::label("[left]", StrokeColor::BodyOnBackground, Cling::Left),
			yard::label("[かいさつぐち]", StrokeColor::BodyOnBackground, Cling::Center),
			yard::label("[right]", StrokeColor::BodyOnBackground, Cling::Right),
		];
		let dark_innards = vec![
			yard::label("[left]", StrokeColor::BodyOnPrimary, Cling::Left),
			yard::label("[かいさつぐち]", StrokeColor::BodyOnPrimary, Cling::Center),
			yard::label("[right]", StrokeColor::BodyOnPrimary, Cling::Right),
		];
		let light_half = cluster(light_innards).pad(1);
		let dark_half = cluster(dark_innards).pad(1).before(yard::fill(FillColor::Primary));
		let rendering = light_half.pack_right(50, dark_half);
		Some(rendering)
	}
}

fn cluster(innards: Vec<ArcYard>) -> ArcYard {
	let arc = yard::trellis(1, 1, Cling::Center, innards);
	arc
}
