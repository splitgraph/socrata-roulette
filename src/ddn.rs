use crate::query_gen::query::{build_dimensions, build_measures, random_query, Query, Syntax};
use crate::query_gen::socrata::{parse_dataset, Column, Dataset, RawDatasetResource};
use gloo_net::http::Request;
use rand::Rng;
use regex::Regex;
use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize)]
struct DDNResponseRow {
    domain: String,
    resource: RawDatasetResource,
}

#[derive(Deserialize)]
struct DDNGetDatasetResponse {
    success: bool,
    rows: Vec<DDNResponseRow>,
}

#[derive(Serialize)]
struct DDNGetDatasetRequest {
    sql: String,
}

#[derive(Serialize)]
struct GQLRequest<T> {
    query: String,
    variables: T,
}

#[derive(Deserialize)]
struct GQLResponse<T> {
    data: T,
    errors: Option<Vec<serde_json::Value>>,
}

#[derive(Serialize)]
struct GQLGetSplitgraphRepoVariables {
    id: String,
    domain: String,
    name: String,
}

const SOCRATA_REPO_QUERY: &str = r#"
query getSocrataRepo($id: String!, $domain: String!, $name: String!) {
    socrataExternalRepositories (datasets:
        [{id: $id, domain: $domain, name: $name}]) {
        namespace
        repository
    }
}"#;

#[derive(Deserialize, Clone)]
pub struct SplitgraphInfo {
    pub namespace: String,
    pub repository: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GQLGetSplitgraphRepoData {
    socrata_external_repositories: Vec<SplitgraphInfo>,
}

pub async fn get_random_dataset() -> Dataset {
    let mut rng = rand::thread_rng();
    let cache_bust: u32 = rng.gen();

    let request = DDNGetDatasetRequest { sql: format!("SELECT * FROM \"splitgraph/socrata\".datasets WHERE {cache_bust} = {cache_bust} ORDER BY random() DESC LIMIT 1")};

    let result = Request::post("https://data.splitgraph.com/sql/query/ddn")
        .json(&request)
        .unwrap()
        .send()
        .await
        .unwrap();

    let parsed_response = result.json::<DDNGetDatasetResponse>().await.unwrap();
    assert!(parsed_response.success);

    let row = parsed_response.rows.first().unwrap();

    parse_dataset(&row.domain, &row.resource)
}

pub async fn get_dataset_namespace_repository(dataset: &Dataset) -> SplitgraphInfo {
    let request = GQLRequest {
        query: SOCRATA_REPO_QUERY.to_string(),
        variables: GQLGetSplitgraphRepoVariables {
            id: dataset.socrata_id.clone(),
            domain: dataset.domain.clone(),
            name: dataset.name.clone(),
        },
    };

    let result = Request::post("https://api.splitgraph.com/gql/cloud/unified/graphql")
        .json(&request)
        .unwrap()
        .send()
        .await
        .unwrap();

    let parsed_response = result
        .json::<GQLResponse<GQLGetSplitgraphRepoData>>()
        .await
        .unwrap();
    assert!(parsed_response.errors.is_none());

    parsed_response
        .data
        .socrata_external_repositories
        .first()
        .unwrap()
        .clone()
}

pub struct SplitgraphDDNSyntax {
    repository: SplitgraphInfo,
}

pub fn slugify_table(table: &str) -> String {
    /// Copy of Splitgraph Socrata loader's table slugifier
    const MAX_LENGTH: usize = 50;

    let re = Regex::new(r"[^\sa-zA-Z0-9]").unwrap();
    let table_lower = table.to_lowercase();
    let replaced = re.replace_all(&table_lower, "");
    let parts: Vec<&str> = replaced.split_whitespace().collect();

    let mut result: String = parts.first().unwrap().to_string();

    for p in parts.iter().skip(1) {
        if result.len() + p.len() + 1 > MAX_LENGTH {
            break;
        };

        result.push('_');
        result.push_str(p)
    }

    result[0..MAX_LENGTH.min(result.len())].to_string()
}

impl Syntax for SplitgraphDDNSyntax {
    fn get_dataset_sql(self: &SplitgraphDDNSyntax, dataset: &Dataset) -> String {
        format!(
            "\"{:}/{:}\".\"{:}\"",
            self.repository.namespace.replace('\"', "\"\""),
            self.repository.repository.replace('\"', "\"\""),
            slugify_table(&dataset.name).replace('\"', "\"\"")
        )
    }

    fn get_column_sql(self: &SplitgraphDDNSyntax, column: &Column) -> String {
        // TODO: fetch real Splitgraph columns and map them
        format!("\"{:}\"", column.pg_name.replace('\"', "\"\""))
    }
}

pub async fn get_random_query_on_random_data(
) -> (Query<SplitgraphDDNSyntax>, Dataset, SplitgraphInfo) {
    let dataset = get_random_dataset().await;

    let namespace_repository = get_dataset_namespace_repository(&dataset).await;

    let measures = build_measures(&dataset);
    let dimensions = build_dimensions(&dataset);

    let query = random_query(
        &dataset,
        &measures,
        &dimensions,
        1..3,
        1..4,
        0..3,
        SplitgraphDDNSyntax {
            repository: namespace_repository.clone(),
        },
    );

    (query, dataset, namespace_repository)
}

#[cfg(test)]
mod tests {
    use crate::ddn::slugify_table;

    #[test]
    fn test_slugify_table() {
        assert_eq!(
            slugify_table("Performance Metrics - Procurement Services - Task Order Request (TOR)"),
            "performance_metrics_procurement_services_task"
        );
        assert_eq!(slugify_table("Some Table"), "some_table");
    }
}
