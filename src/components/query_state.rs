use crate::ddn::SplitgraphInfo;
use crate::{Dataset, Query, Syntax};
use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Eq, PartialEq)]
pub struct QueryState {
    pub query: String,
    pub dataset_id: String,
    pub dataset_name: String,
    pub dataset_domain: String,
    pub sg_namespace: String,
    pub sg_repository: String,
}

impl QueryState {
    pub fn from_query_dataset<T: Syntax>(
        query: &Query<T>,
        dataset: &Dataset,
        splitgraph: &SplitgraphInfo,
    ) -> Self {
        Self {
            query: query.to_sql(),
            dataset_id: dataset.socrata_id.clone(),
            dataset_name: dataset.name.clone(),
            dataset_domain: dataset.domain.clone(),
            sg_namespace: splitgraph.namespace.clone(),
            sg_repository: splitgraph.repository.clone(),
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
pub enum QueryGenerationState {
    None,
    GeneratingQuery,
    Ready,
}

#[derive(Clone, Eq, PartialEq)]
pub enum ComponentQueryState {
    None,
    GeneratingQuery,
    Ready(QueryState),
}
