use crate::components::query_state::ComponentQueryState;
use percent_encoding::{utf8_percent_encode, CONTROLS};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct SplitgraphEmbedQueryProps {
    pub state: ComponentQueryState,
}

#[function_component(SplitgraphEmbedQuery)]
pub fn splitgraph_embed_query(
    SplitgraphEmbedQueryProps { state }: &SplitgraphEmbedQueryProps,
) -> Html {
    html! {
        <div class={classes!("flex", "rounded-sm", "bg-[#F4F9FF]", "align-middle", "items-center", "justify-center", "w-full", "h-full", "my-8", "h-[40rem]")}>
            {
                match state {
                    ComponentQueryState::None => html! {
                        <span class={classes!("inline-block", "align-middle", "font-bold")}>{"Ready to go!"}</span>
                    },
                    ComponentQueryState::GeneratingQuery => html! {
                        <span class={classes!("inline-block", "align-middle", "font-bold")}>{"Please wait..."}</span>
                    },
                    ComponentQueryState::Ready(q) => {
                        let iframe_target = format!(
                            "https://www.splitgraph.com/embed/workspace/ddn?layout=hsplit&query={:}",
                            utf8_percent_encode(&q.query, CONTROLS)
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
