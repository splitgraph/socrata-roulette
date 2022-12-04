use crate::components::query_state::ComponentQueryState;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct DatasetInfoProps {
    pub state: ComponentQueryState,
}

#[function_component(DatasetInfo)]
pub fn dataset_info(DatasetInfoProps { state }: &DatasetInfoProps) -> Html {
    html! {
        <div class={classes!("w-full", "h-full", "my-8")}>
            {
                match state {
                    ComponentQueryState::None | ComponentQueryState::GeneratingQuery => html! {},
                    ComponentQueryState::Ready(q) => {
                        let domain_url = format!("https://{:}", q.dataset_domain);
                        let splitgraph_url = format!("https://www.splitgraph.com/{:}/{:}", q.sg_namespace, q.sg_repository);
                        // Socrata permalinks all seem to have a similar format
                        let dataset_url = format!("https://{:}/d/{:}", q.dataset_domain, q.dataset_id);
                        let socrata_button_text = format!("Socrata: {:}", q.dataset_id);

                        html! {
                            <>
                            <h3 class={classes!("text-2xl", "font-bold", "mt-0", "mb-8", "text-slate-200")}>{ q.dataset_name.clone() }</h3>

                            <div class={classes!("grid", "grid-cols-1", "sm:grid-cols-3", "gap-4", "text-center")}>
                                <a href={ splitgraph_url } target={ "_blank" } class={classes!("bg-slate-300", "hover:bg-slate-400", "py-2", "px-4", "rounded")}>{ "View on Splitgraph" }</a>
                                <a href={ domain_url } target={ "_blank" } class={classes!("bg-slate-300", "hover:bg-slate-400", "py-2", "px-4", "rounded")}>{ q.dataset_domain.clone() }</a>
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
