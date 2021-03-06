use yui::prelude::*;
use yui::prelude::yard::ButtonState;
use yui::SenderLink;
use yui::sparks::selection_editor::SelectionEditorSpark;
use yui::palette::FillGrade::Plain;

#[derive(Debug)]
pub struct ButtonDemo {}

#[derive(Copy, Clone)]
pub enum State { Beavis, Hall }

pub enum Action {
    OfferChoice,
    Choose(Option<usize>),
}

impl Spark for ButtonDemo {
    type State = State;
    type Action = Action;
    type Report = usize;

    fn create(&self, _ctx: &Create<Self::Action, Self::Report>) -> Self::State { State::Beavis }

    fn flow(&self, action: Self::Action, ctx: &impl Flow<Self::State, Self::Action, Self::Report>) -> AfterFlow<Self::State, Self::Report> {
        match action {
            Action::OfferChoice => {
                let choices = vec!["Beavis", "Hall  "];
                let selected = match ctx.state() {
                    State::Beavis => 0,
                    State::Hall => 1,
                };
                let spark = SelectionEditorSpark { selected, choices };
                ctx.start_prequel(spark, ctx.link().map(|it: Option<(usize, &'static str)>| {
                    let choice = it.map(|(i, _)| i);
                    Action::Choose(choice)
                }));
                AfterFlow::Ignore
            }
            Action::Choose(choice) => {
                if let Some(choice) = choice {
                    let state = if choice == 0 { State::Beavis } else { State::Hall };
                    AfterFlow::Revise(state)
                } else {
                    AfterFlow::Ignore
                }
            }
        }
    }

    fn render(state: &Self::State, link: &SenderLink<Self::Action>) -> Option<ArcYard> {
        let (text1, text2) = match state {
            State::Beavis => ("Beavis".to_string(), "Butthead".to_string()),
            State::Hall => ("Hall".to_string(), "Oates".to_string()),
        };
        let dark_half =
            yard::trellis(1, 1, Cling::Center, vec![
                yard::button(text1, ButtonState::enabled(link.map(|_| Action::OfferChoice))),
                yard::button(text2, ButtonState::default(SenderLink::new_f(|_| info!("Butthead")))),
            ])
                .pad(3).before(yard::fill(FillColor::Primary, Plain));
        let light_half =
            yard::trellis(1, 1, Cling::Center, vec![
                yard::button("Garfunkel", ButtonState::enabled(SenderLink::new_f(|_| info!("Garfunkel")))),
                yard::button("Simon", ButtonState::enabled(SenderLink::new_f(|_| info!("Simon")))),
            ]).pad(3).before(yard::fill(FillColor::Background, Plain));
        let full = light_half.pack_right(40, dark_half);
        Some(full)
    }
}