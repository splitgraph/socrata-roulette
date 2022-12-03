mod ddn;
mod query_gen;

use crate::ddn::get_random_query_on_random_data;
use crate::query_gen::socrata::Dataset;
use percent_encoding::{utf8_percent_encode, CONTROLS};
use yew::prelude::*;

#[derive(Clone, Eq, PartialEq)]
enum SplitgraphEmbedQueryState {
    None,
    GeneratingQuery,
    Query(String, Dataset),
}

#[derive(Properties, PartialEq)]
struct SplitgraphEmbedQueryProps {
    state: SplitgraphEmbedQueryState,
}

#[function_component(SplitgraphEmbedQuery)]
fn splitgraph_embed_query(SplitgraphEmbedQueryProps { state }: &SplitgraphEmbedQueryProps) -> Html {
    html! {
        <div class={classes!("flex", "rounded-sm", "bg-[#F4F9FF]", "align-middle", "items-center", "justify-center", "w-full", "h-full", "my-8", "h-[40rem]")}>
            {
                match state {
                    SplitgraphEmbedQueryState::None => html! {
                        <span class={classes!("inline-block", "align-middle", "font-bold")}>{"Ready to go!"}</span>
                    },
                    SplitgraphEmbedQueryState::GeneratingQuery => html! {
                        <span class={classes!("inline-block", "align-middle", "font-bold")}>{"Please wait..."}</span>
                    },
                    SplitgraphEmbedQueryState::Query(q, _) => {
                        let iframe_target = format!(
                            "https://www.splitgraph.com/embed/workspace/ddn?layout=hsplit&query={:}",
                            utf8_percent_encode(q, CONTROLS)
                        );

                        html! {
                            <iframe class={classes!("w-full", "h-full")} src={iframe_target} />
                        }
                    }
                }
            }
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct DatasetInfoProps {
    state: SplitgraphEmbedQueryState,
}

#[function_component(DatasetInfo)]
fn dataset_info(DatasetInfoProps { state }: &DatasetInfoProps) -> Html {
    html! {
        <div class={classes!("w-full", "h-full", "my-8")}>
            {
                match state {
                    SplitgraphEmbedQueryState::None | SplitgraphEmbedQueryState::GeneratingQuery => html! {},
                    SplitgraphEmbedQueryState::Query(_, d) => {
                        let domain_url = format!("https://{:}", d.domain);
                        // TODO: kinda giant hack that relies on Socrata redirecting properly at this ID
                        let dataset_url = format!("https://{:}/_/_/{:}", d.domain, d.socrata_id);
                        let socrata_button_text = format!("Socrata: {:}", d.socrata_id);

                        html! {
                            <>
                            <h3 class={classes!("text-2xl", "font-bold", "mt-0", "mb-8", "text-slate-200")}>{ d.name.clone() }</h3>

                            <div class={classes!("flex", "space-x-4")}>
                                <a href={ domain_url } target={ "_blank" } class={classes!("bg-slate-300", "hover:bg-slate-400", "py-2", "px-4", "rounded")}>{ d.domain.clone() }</a>
                                <a href={ dataset_url } target={ "_blank" } class={classes!("bg-slate-300", "hover:bg-slate-400", "py-2", "px-4", "rounded")}>{ socrata_button_text }</a>
                            </div>
                            </>
                        }
                    }
                }
            }
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct RandomQueryButtonProps {
    onclick: Callback<MouseEvent>,
    state: SplitgraphEmbedQueryState,
}

#[function_component(RandomQueryButton)]
fn random_query_button(RandomQueryButtonProps { onclick, state }: &RandomQueryButtonProps) -> Html {
    match state {
        SplitgraphEmbedQueryState::None => html! {
            <button class={classes!("bg-slate-300", "hover:bg-slate-400", "font-bold", "py-2", "px-4", "rounded")} {onclick}>{ "I'm feeling lucky!" }</button>
        },
        SplitgraphEmbedQueryState::GeneratingQuery => html! {
            <button class={classes!("bg-slate-300", "hover:bg-slate-400", "font-bold", "py-2", "px-4", "rounded", "opacity-50", "cursor-not-allowed")}>{ "Generating..." }</button>
        },
        SplitgraphEmbedQueryState::Query(_, _) => html! {
            <button class={classes!("bg-slate-300", "hover:bg-slate-400", "font-bold", "py-2", "px-4", "rounded")} {onclick}>{ "Another!" }</button>
        },
    }
}

#[function_component(App)]
fn app() -> Html {
    let query_state = use_state(|| SplitgraphEmbedQueryState::None);

    let onclick = {
        let query_state = query_state.clone();
        Callback::from(move |_| {
            let query_state = query_state.clone();
            wasm_bindgen_futures::spawn_local(async move {
                query_state.set(SplitgraphEmbedQueryState::GeneratingQuery);
                let (query, dataset) = get_random_query_on_random_data().await;
                query_state.set(SplitgraphEmbedQueryState::Query(query.to_sql(), dataset));
            });
        })
    };

    // TODO: fix mobile
    // TODO: some error handling
    // TODO: factor some styles out
    // TODO: add a way to generate a nice query name
    // TODO: easier way to copy the URL
    //       (grab more stuff out of the Socrata result, incl. the real original URL)

    html! {
        <>
            <div class={classes!("flex", "flex-col", "min-h-screen", "bg-slate-900")}>
                <div class={classes!("container", "mx-auto", "max-w-3xl")}>
                    <section class={classes!("text-center", "my-8")}>
                        <h1 class={classes!("text-5xl", "font-bold", "mt-0", "mb-6", "text-slate-200")}>{ "Socrata Roulette" }</h1>
                        <h3 class={classes!("text-2xl", "font-bold", "mt-0", "mb-8", "text-slate-200")}>{ "Run a random SQL query on a random open government dataset" }</h3>
                        <RandomQueryButton onclick={onclick} state={ (*query_state).clone() } />
                        <SplitgraphEmbedQuery state={ (*query_state).clone() } />
                    </section>
                    <section class={classes!("my-8")}>
                        <DatasetInfo state={ (*query_state).clone() } />
                    </section>
                </div>
                <footer class={classes!("mt-auto", "border-t", "shadow", "md:flex", "md:items-center", "md:justify-between", "md:p-6", "bg-slate-800", "border-slate-600")}>
                    <div class={classes!("container", "mx-auto", "max-w-3xl")}>
                    <span class={classes!("text-sm", "text-gray-500", "sm:text-center")}>{"Powered by "}<a class={classes!("text-blue-500", "hover:underline")} href={ "https://www.splitgraph.com/explore" }>{"Splitgraph"}</a>{"."}</span>
                    </div>
                </footer>
            </div>
        </>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
