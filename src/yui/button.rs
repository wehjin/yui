use std::rc::Rc;

use crate::yui::{Cling, Focus, FocusType, RenderContext, Yard, YardOption};
use crate::yui::fill::fill_yard;
use crate::yui::label::label_yard;
use crate::yui::layout::LayoutContext;
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

	fn update(&self, _option: YardOption) {}

	fn layout(&self, ctx: &mut LayoutContext) -> usize {
		let (edge_index, edge_bounds) = ctx.edge_bounds();
		self.fill_yard.layout(ctx);
		self.label_yard.layout(ctx);
		ctx.add_focus(Focus {
			yard_id: self.id(),
			focus_type: FocusType::Submit,
			bounds: edge_bounds,
		});
		edge_index
	}

	fn render(&self, ctx: &dyn RenderContext) {
		let focus_id = ctx.focus_id();
		let fill_color = if focus_id == self.id {
			FillColor::BackgroundWithFocus
		} else {
			FillColor::Background
		};
		self.fill_yard.update(YardOption::FillColor(fill_color));
		self.fill_yard.render(ctx);
		self.label_yard.render(ctx);
	}
}
