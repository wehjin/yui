use std::rc::Rc;

use crate::yui::{Cling, LayoutContext, RenderContext, Yard};
use crate::yui::fill::fill_yard;
use crate::yui::label::label_yard;
use crate::yui::palette::{FillColor, StrokeColor};

pub fn button_yard(text: &str) -> Rc<dyn Yard> {
	ButtonYard::new(text)
}

struct ButtonYard {
	id: i32,
	label_yard: Rc<dyn Yard>,
	fill_yard: Rc<dyn Yard>,
}

impl ButtonYard {
	fn new(text: &str) -> Rc<dyn Yard> {
		Rc::new(ButtonYard {
			id: rand::random(),
			label_yard: label_yard(
				&text.to_uppercase(),
				StrokeColor::EnabledOnBackground,
				Cling::CenterMiddle,
			),
			fill_yard: fill_yard(FillColor::BackgroundWithFocus),
		})
	}
}

impl Yard for ButtonYard {
	fn id(&self) -> i32 {
		self.id
	}

	fn layout(&self, ctx: &mut dyn LayoutContext) -> usize {
		let (edge_index, _edge_bounds) = ctx.edge_bounds();
		self.fill_yard.layout(ctx);
		self.label_yard.layout(ctx);
		edge_index
	}

	fn render(&self, ctx: &dyn RenderContext) {
		self.fill_yard.render(ctx);
		self.label_yard.render(ctx);
	}
}
