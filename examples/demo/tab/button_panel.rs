use yui::prelude::*;
use yui::prelude::yard::ButtonState;

#[derive(Debug)]
pub struct ButtonDemo {}

impl Spark for ButtonDemo {
	type State = ();
	type Action = ();
	type Report = usize;

	fn create(&self, _ctx: &Create<Self::Action, Self::Report>) -> Self::State { () }

	fn flow(&self, _action: Self::Action, _ctx: &impl Flow<Self::State, Self::Action, Self::Report>) -> AfterFlow<Self::State, Self::Report> {
		AfterFlow::Ignore
	}

	fn render(_state: &Self::State, _link: &Link<Self::Action>) -> Option<ArcYard> {
		let dark_half =
			yard::trellis(1, 1, Cling::Center, vec![
				yard::button("Beavis", ButtonState::enabled(|_| info!("Beavis"))),
				yard::button("Butthead", ButtonState::default(|_| info!("Butthead"))),
			]).pad(3).before(yard::fill(FillColor::Primary));
		let light_half =
			yard::trellis(1, 1, Cling::Center, vec![
				yard::button("Garfunkel", ButtonState::enabled(|_| info!("Garfunkel"))),
				yard::button("Simon", ButtonState::enabled(|_| info!("Simon"))),
			]).pad(3).before(yard::fill(FillColor::Background));
		let full = light_half.pack_right(40, dark_half);
		Some(full)
	}
}