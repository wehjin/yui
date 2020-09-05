use yui::prelude::*;

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
		let dark_fill = yard::fill(FillColor::Primary);
		let buttons = yard::trellis(1, 1, Cling::Center, vec![
			yard::button_enabled("Simon", |_| {}),
			yard::button_enabled("Garfunkel", |_| {})
		]);
		let dark_half = buttons.pad(3).before(dark_fill);
		let full = yard::empty().pack_right(40, dark_half);
		Some(full.pad(2))
	}
}