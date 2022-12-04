use crate::components::query_state::ComponentQueryState;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct RandomQueryButtonProps {
    pub onclick: Callback<MouseEvent>,
    pub state: ComponentQueryState,
}

#[function_component(RandomQueryButton)]
pub fn random_query_button(
    RandomQueryButtonProps { onclick, state }: &RandomQueryButtonProps,
) -> Html {
    match state {
        ComponentQueryState::None => html! {
            <button class={classes!("bg-slate-300", "hover:bg-slate-400", "font-bold", "py-2", "px-4", "rounded")} {onclick}>{ "I'm feeling lucky!" }</button>
        },
        ComponentQueryState::GeneratingQuery => html! {
            <button class={classes!("bg-slate-300", "hover:bg-slate-400", "font-bold", "py-2", "px-4", "rounded", "opacity-50", "cursor-not-allowed")}>{ "Generating..." }</button>
        },
        ComponentQueryState::Ready(_) => html! {
            <button class={classes!("bg-slate-300", "hover:bg-slate-400", "font-bold", "py-2", "px-4", "rounded")} {onclick}>{ "Another!" }</button>
        },
    }
}
