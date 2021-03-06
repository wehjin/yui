use std::ops::Deref;
use std::sync::{Arc, RwLock};

use crate::{ArcYard, Before, Cling, Focus, FocusType, Link, Pack, Place, render_submit, RenderContext, SenderLink, SyncLink, yard};
use crate::layout::LayoutContext;
use crate::palette::{FillColor, StrokeColor, FillGrade};
use crate::yard::{ignore_touch, Yard, YardOption};

pub use self::tab::*;

mod tab;

pub fn tabbar(
    tabs: &[impl Tab],
    selected_index: usize,
    on_select: SenderLink<usize>,
) -> ArcYard {
    let tabs: Vec<(i32, ArcYard)> = tabs.iter().enumerate().map({
        let on_select = on_select.clone();
        move |(index, tab)| {
            let id = tab.uid();
            let label = tab.label();
            let tab_width = (label.chars().count() + 2 * 2) as i32;
            let tab_on_select = on_select.clone().map(move |_| index);
            let tab_yard = tab_yard(id, label, index, selected_index, tab_on_select);
            (tab_width, tab_yard)
        }
    }).collect();
    let (width, bar) = tabs.into_iter()
        .fold((0, yard::empty()), |(bar_width, bar), (width, tab)| {
            (bar_width + width, bar.pack_right(width, tab))
        });
    let centered_bar = bar.place_center(width);
    let fill = yard::fill(FillColor::Primary, FillGrade::Plain);
    centered_bar.before(fill)
}


fn tab_yard(id: i32, label: &str, index: usize, active_index: usize, select: SenderLink<()>) -> ArcYard {
    let label = yard::label(label, StrokeColor::EnabledOnPrimary, Cling::Center);
    let underline = yard::glyph(StrokeColor::EnabledOnPrimary, move || if index == active_index { '_' } else { '\0' });
    let content = yard::empty().pack_bottom(1, label).pack_bottom(1, underline);
    let is_pressed = Arc::new(RwLock::new(false));
    Arc::new(TabYard { id, content, is_pressed, select: select.into() })
}

struct TabYard {
    id: i32,
    content: ArcYard,
    is_pressed: Arc<RwLock<bool>>,
    select: SyncLink<()>,
}

impl TabYard {
    fn is_pressed(&self) -> bool {
        let is_pressed = self.is_pressed.read().expect("read is_pressed").deref().to_owned();
        is_pressed
    }
}

impl Yard for TabYard {
    fn render(&self, ctx: &dyn RenderContext) {
        let (row, col) = ctx.spot();
        let bounds = ctx.yard_bounds(self.id);
        if bounds.intersects(row, col) {
            let fill_grade = if ctx.focus_id() == self.id() {
                if self.is_pressed() {
                    FillGrade::Press
                } else {
                    FillGrade::Focus
                }
            } else {
                FillGrade::Plain
            };
            ctx.set_fill_grade(fill_grade, bounds.z);
            ctx.set_fill(FillColor::Primary, bounds.z);
            self.content.render(ctx);
        }
    }
    fn layout(&self, ctx: &mut LayoutContext) -> usize {
        let (bounds_id, bounds) = ctx.edge_bounds();
        self.content.layout(ctx);
        ctx.set_yard_bounds(self.id(), bounds_id);
        let is_pressed = self.is_pressed.clone();
        ctx.add_focus(Focus {
            yard_id: self.id(),
            focus_type: FocusType::Submit,
            bounds,
            priority: 0,
            action_block: Arc::new({
                let on_select = self.select.clone();
                move |ctx| {
                    render_submit(&is_pressed, ctx, &ignore_touch());
                    on_select.send(());
                }
            }),
        });
        bounds_id
    }
    fn update(&self, _option: YardOption) {}

    fn id(&self) -> i32 { self.id }
}
