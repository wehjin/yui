use std::sync::{Arc, RwLock};

use crate::palette::{FillColor, FillGrade};
use crate::RenderContext;
use crate::yard::{ArcYard, Yard, YardOption};
use crate::yui::layout::LayoutContext;

pub fn fill(color: FillColor, grade: FillGrade) -> ArcYard {
    //! Produce a yard that renders as a rectangle filled the specified color.
    Arc::new(FillYard {
        id: rand::random(),
        color: RwLock::new((color, grade)),
    })
}

struct FillYard {
    id: i32,
    color: RwLock<(FillColor, FillGrade)>,
}

impl Yard for FillYard {
    fn render(&self, ctx: &dyn RenderContext) {
        let (row, col) = ctx.spot();
        let bounds = ctx.yard_bounds(self.id);
        if bounds.intersects(row, col) {
            let (color, grade) = *self.color.read().expect("read color");
            ctx.set_fill_grade(grade, bounds.z);
            ctx.set_fill(color, bounds.z);
        }
    }

    fn layout(&self, ctx: &mut LayoutContext) -> usize {
        let (bounds_id, _bounds) = ctx.edge_bounds();
        ctx.set_yard_bounds(self.id, bounds_id);
        bounds_id
    }

    fn update(&self, option: YardOption) {
        let YardOption::FillColor(color, grade) = option;
        *self.color.write().expect("write color") = (color, grade);
    }

    fn id(&self) -> i32 { self.id }
}