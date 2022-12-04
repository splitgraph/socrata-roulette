use yew::prelude::*;
use yew_router::history::HistoryResult;
use yew_router::prelude::use_location;

use crate::components::dataset_info::DatasetInfo;
use crate::components::query_button::RandomQueryButton;
use crate::components::query_embed::SplitgraphEmbedQuery;
use crate::components::query_state::ComponentQueryState;
use crate::{QueryGenerationState, QueryState};

#[derive(Properties, PartialEq)]
pub struct RandomQueryProps {
    pub onclick: Callback<MouseEvent>,
    pub state: QueryGenerationState,
}

#[function_component(RandomQuery)]
pub fn random_query(RandomQueryProps { onclick, state }: &RandomQueryProps) -> Html {
    // Compute the component state based on the state that was passed to us and the
    // current location (that may contain the query)
    let location = use_location().unwrap();
    let query_state: HistoryResult<QueryState> = location.query();

    let component_state = match (state, query_state) {
        // On first load, if we managed to deserialize the location, pass the query to the children
        (QueryGenerationState::None, Ok(qs)) |
        // Same if we actually managed to generate the query
        (QueryGenerationState::Ready, Ok(qs))  => ComponentQueryState::Ready(qs),
        (QueryGenerationState::GeneratingQuery, _) => ComponentQueryState::GeneratingQuery,
        (_, Err(_)) => ComponentQueryState::None,
    };

    html! {
        <div class={classes!("my-4")}>
            <div class={classes!("text-center")}>
                <RandomQueryButton onclick={onclick} state={ component_state.clone() } />
            </div>
            <SplitgraphEmbedQuery state={ component_state.clone() } />
            <DatasetInfo state={ component_state.clone() } />
        </div>
    }
}
