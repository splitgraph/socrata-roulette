mod components;
mod ddn;
mod query_gen;

use crate::components::random_query::RandomQuery;
use crate::ddn::get_random_query_on_random_data;
use crate::query_gen::socrata::Dataset;
use std::collections::HashMap;

use crate::components::query_state::QueryState;
use crate::query_gen::query::{Query, Syntax};
use components::query_state::QueryGenerationState;
use yew::prelude::*;
use yew_router::prelude::*;
use yew_router::AnyRoute;

#[function_component(Main)]
fn main_app() -> Html {
    let query_gen_state = use_state(|| QueryGenerationState::None);

    let onclick = {
        let query_gen_state = query_gen_state.clone();
        let navigator = use_navigator().unwrap();
        Callback::from(move |_| {
            let query_gen_state = query_gen_state.clone();
            let navigator = navigator.clone();
            wasm_bindgen_futures::spawn_local(async move {
                query_gen_state.set(QueryGenerationState::GeneratingQuery);
                let (query, dataset, splitgraph) = get_random_query_on_random_data().await;

                let query_state = QueryState::from_query_dataset(&query, &dataset, &splitgraph);
                let params: HashMap<&str, &str> = HashMap::new();
                let route = AnyRoute::from_path("", &params).unwrap();
                navigator.push_with_query(&route, &query_state).unwrap();

                query_gen_state.set(QueryGenerationState::Ready)
            });
        })
    };

    // TODO: some error handling
    // TODO: factor some styles out
    // TODO: add a way to generate a nice query name
    // TODO: easier way to copy the URL
    //       (grab more stuff out of the Socrata result, incl. the real original URL)

    html! {
        <div class={classes!("flex", "flex-col", "min-h-screen", "bg-slate-900")}>
            <div class={classes!("container", "mx-auto", "max-w-3xl", "p-2")}>
                <section class={classes!("text-center", "my-8")}>
                    <h1 class={classes!("text-5xl", "font-bold", "mt-0", "mb-6", "text-slate-200")}>{ "Socrata Roulette" }</h1>
                    <h3 class={classes!("text-2xl", "font-bold", "mt-0", "mb-8", "text-slate-200")}>{ "Run a random SQL query on a random open government dataset" }</h3>
                </section>
                <RandomQuery onclick={onclick} state={ (*query_gen_state).clone() } />
            </div>
            <footer class={classes!("mt-auto", "border-t", "shadow", "md:flex", "md:items-center", "md:justify-between", "p-2", "md:p-6", "bg-slate-800", "border-slate-600")}>
                <div class={classes!("container", "mx-auto", "max-w-3xl")}>
                <span class={classes!("text-sm", "text-gray-500", "sm:text-center")}>{"Powered by "}<a class={classes!("text-blue-500", "hover:underline")} href={ "https://www.splitgraph.com/explore" }>{"Splitgraph"}</a>{"."}</span>
                </div>
            </footer>
        </div>
    }
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <Main/>
        </BrowserRouter>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
