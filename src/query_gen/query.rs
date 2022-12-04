use crate::query_gen::socrata::{Column, DataType, Dataset};
use itertools::Itertools;
use rand::seq::SliceRandom;
use rand::Rng;
use std::ops::Range;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum MeasureType {
    Count,
    Sum,
    Average,
    Min,
    Max,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Measure {
    pub type_: MeasureType,
    pub column: Option<Column>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Dimension {
    // No need to specify a dimension type since we can't use functions
    // in GROUP BY clauses right now
    pub column: Column,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum OrderByItem {
    Measure(Measure),
    Dimension(Dimension),
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum OrderByDirection {
    Asc,
    Desc,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct OrderBy {
    pub item: OrderByItem,
    pub direction: OrderByDirection,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Query<T: Syntax> {
    pub dataset: Dataset,
    pub measures: Vec<Measure>,
    pub dimensions: Vec<Dimension>,
    pub order_by: Vec<OrderBy>,
    pub limit: Option<usize>,

    syntax: T,
}

pub trait Syntax {
    /// Get an SQL identifier for a dataset
    fn get_dataset_sql(&self, dataset: &Dataset) -> String;
    /// Get an SQL identifier for a column
    fn get_column_sql(&self, column: &Column) -> String;
}

pub struct DefaultSyntax {}

impl Syntax for DefaultSyntax {
    fn get_dataset_sql(&self, dataset: &Dataset) -> String {
        dataset.socrata_id.clone()
    }

    fn get_column_sql(&self, column: &Column) -> String {
        column.pg_name.clone()
    }
}

impl<T: Syntax> Query<T> {
    fn emit_measure(&self, measure: &Measure, include_alias: bool) -> String {
        let func = match measure.type_ {
            MeasureType::Count => "COUNT",
            MeasureType::Sum => "SUM",
            MeasureType::Average => "AVG",
            MeasureType::Min => "MIN",
            MeasureType::Max => "MAX",
        };

        if measure.type_ == MeasureType::Count {
            return "COUNT(*)".to_string();
        }

        let column_sql = self.syntax.get_column_sql(measure.column.as_ref().unwrap());
        // TODO: use the same self.syntax route for unquoted columns
        let column_unquoted = &measure.column.as_ref().unwrap().pg_name;

        let mut result = format!("{func}({column_sql})");

        if include_alias {
            result.push_str(format!(" AS {0}_{1}", func.to_lowercase(), column_unquoted).as_str());
        }

        result
    }

    fn emit_dimension(&self, dimension: &Dimension) -> String {
        self.syntax.get_column_sql(&dimension.column)
    }

    fn emit_order_by(&self, order_by: &OrderBy) -> String {
        let mut result = String::new();

        result.push_str(
            match &order_by.item {
                OrderByItem::Measure(m) => self.emit_measure(m, false),
                OrderByItem::Dimension(d) => self.emit_dimension(d),
            }
            .as_str(),
        );

        result.push_str(match &order_by.direction {
            OrderByDirection::Asc => " ASC",
            OrderByDirection::Desc => " DESC",
        });

        result
    }

    pub fn to_sql(&self) -> String {
        let mut result = String::new();

        result.push_str("SELECT\n  ");

        result.push_str(
            self.dimensions
                .iter()
                .map(|d| self.emit_dimension(d))
                .chain(self.measures.iter().map(|m| self.emit_measure(m, true)))
                .join(",\n  ")
                .as_str(),
        );

        result.push_str("\nFROM ");
        result.push_str(self.syntax.get_dataset_sql(&self.dataset).as_str());

        if !self.dimensions.is_empty() {
            result.push_str("\nGROUP BY\n  ");
            result.push_str(
                self.dimensions
                    .iter()
                    .map(|d| self.emit_dimension(d))
                    .join(",\n  ")
                    .as_str(),
            )
        }

        if !self.order_by.is_empty() {
            result.push_str("\nORDER BY\n  ");
            result.push_str(
                self.order_by
                    .iter()
                    .map(|o| self.emit_order_by(o))
                    .join(",\n  ")
                    .as_str(),
            )
        }

        if let Some(l) = self.limit {
            result.push_str("\nLIMIT ");
            result.push_str(l.to_string().as_str());
        }

        result
    }
    pub fn new(
        dataset: Dataset,
        measures: Vec<Measure>,
        dimensions: Vec<Dimension>,
        order_by: Vec<OrderBy>,
        limit: Option<usize>,
        syntax: T,
    ) -> Self {
        Self {
            dataset,
            measures,
            dimensions,
            order_by,
            limit,
            syntax,
        }
    }
}

pub fn build_measures(dataset: &Dataset) -> Vec<Measure> {
    // COUNT(*) always exists
    let mut measures = vec![Measure {
        type_: MeasureType::Count,
        column: None,
    }];

    for column in &dataset.columns {
        // Ignore Socrata derived geo columns
        if column.pg_name.contains(":@computed_region") {
            continue;
        };

        if matches!(
            column.data_type,
            DataType::Money
                | DataType::Number
                | DataType::Double
                | DataType::FloatingTimestamp
                | DataType::CalendarDate
        ) {
            measures.push(Measure {
                type_: MeasureType::Min,
                column: Some(column.clone()),
            });
            measures.push(Measure {
                type_: MeasureType::Max,
                column: Some(column.clone()),
            });
        };

        if matches!(
            column.data_type,
            DataType::Money | DataType::Number | DataType::Double
        ) {
            measures.push(Measure {
                type_: MeasureType::Sum,
                column: Some(column.clone()),
            });
            measures.push(Measure {
                type_: MeasureType::Average,
                column: Some(column.clone()),
            });
        };
    }

    measures
}

pub fn build_dimensions(dataset: &Dataset) -> Vec<Dimension> {
    // possible dimensions:
    //  - all text cols
    //  - all dates (though we can't even extract a date part)

    dataset
        .columns
        .iter()
        .filter_map(|c| match c.data_type {
            DataType::Text | DataType::Checkbox | DataType::Url => {
                Some(Dimension { column: c.clone() })
            }
            _ => None,
        })
        .collect()
}

pub fn random_query<T: Syntax>(
    dataset: &Dataset,
    measures: &Vec<Measure>,
    dimensions: &Vec<Dimension>,
    no_dimensions: Range<usize>,
    no_measures: Range<usize>,
    no_order_bys: Range<usize>,
    syntax: T,
) -> Query<T> {
    let mut rng = rand::thread_rng();

    let no_dimensions = rng.gen_range(no_dimensions);
    let no_measures = rng.gen_range(no_measures);
    let no_order_bys = rng.gen_range(no_order_bys).min(no_dimensions + no_measures);

    let chosen_measures: Vec<Measure> = measures
        .choose_multiple(&mut rng, no_measures)
        .cloned()
        .collect();
    let chosen_dimensions: Vec<Dimension> = dimensions
        .choose_multiple(&mut rng, no_dimensions)
        .cloned()
        .collect();

    let mut all_order_bys: Vec<OrderBy> = chosen_measures
        .iter()
        .map(|m| OrderBy {
            item: OrderByItem::Measure(m.clone()),
            direction: match rng.gen_range(0..2) {
                0 => OrderByDirection::Asc,
                _ => OrderByDirection::Desc,
            },
        })
        .collect();

    all_order_bys.extend(chosen_dimensions.iter().map(|d| OrderBy {
        item: OrderByItem::Dimension(d.clone()),
        direction: match rng.gen_range(0..2) {
            0 => OrderByDirection::Asc,
            _ => OrderByDirection::Desc,
        },
    }));

    let chosen_order_bys = all_order_bys
        .choose_multiple(&mut rng, no_order_bys)
        .cloned()
        .collect();

    Query::new(
        dataset.clone(),
        chosen_measures,
        chosen_dimensions,
        chosen_order_bys,
        Some(100),
        syntax,
    )
}

#[cfg(test)]
mod tests {
    use crate::query_gen::query::{
        build_dimensions, build_measures, random_query, DefaultSyntax, Dimension, Measure,
        MeasureType, OrderBy, OrderByDirection, OrderByItem, Query,
    };
    use crate::query_gen::socrata::{parse_dataset, Column, DataType, Dataset, RawDatasetResource};
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn test_build_measures() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/test/socrata_resource.json");
        let resource_json = fs::read_to_string(d).unwrap();
        let resource: RawDatasetResource = serde_json::from_str(&resource_json).unwrap();
        let dataset = parse_dataset("data.edmonton.ca", &resource);

        assert_eq!(
            build_measures(&dataset),
            vec![
                Measure {
                    type_: MeasureType::Count,
                    column: None
                },
                Measure {
                    type_: MeasureType::Min,
                    column: Some(Column {
                        human_name: "Facility/Service ID".to_string(),
                        pg_name: "facility_service_id".to_string(),
                        data_type: DataType::Number
                    })
                },
                Measure {
                    type_: MeasureType::Max,
                    column: Some(Column {
                        human_name: "Facility/Service ID".to_string(),
                        pg_name: "facility_service_id".to_string(),
                        data_type: DataType::Number
                    })
                },
                Measure {
                    type_: MeasureType::Sum,
                    column: Some(Column {
                        human_name: "Facility/Service ID".to_string(),
                        pg_name: "facility_service_id".to_string(),
                        data_type: DataType::Number
                    })
                },
                Measure {
                    type_: MeasureType::Average,
                    column: Some(Column {
                        human_name: "Facility/Service ID".to_string(),
                        pg_name: "facility_service_id".to_string(),
                        data_type: DataType::Number
                    })
                },
                Measure {
                    type_: MeasureType::Min,
                    column: Some(Column {
                        human_name: "Notification ID".to_string(),
                        pg_name: "notification_id".to_string(),
                        data_type: DataType::Number
                    })
                },
                Measure {
                    type_: MeasureType::Max,
                    column: Some(Column {
                        human_name: "Notification ID".to_string(),
                        pg_name: "notification_id".to_string(),
                        data_type: DataType::Number
                    })
                },
                Measure {
                    type_: MeasureType::Sum,
                    column: Some(Column {
                        human_name: "Notification ID".to_string(),
                        pg_name: "notification_id".to_string(),
                        data_type: DataType::Number
                    })
                },
                Measure {
                    type_: MeasureType::Average,
                    column: Some(Column {
                        human_name: "Notification ID".to_string(),
                        pg_name: "notification_id".to_string(),
                        data_type: DataType::Number
                    })
                },
                Measure {
                    type_: MeasureType::Min,
                    column: Some(Column {
                        human_name: "Notice Publish Start Date".to_string(),
                        pg_name: "notice_publish_start_date".to_string(),
                        data_type: DataType::CalendarDate
                    })
                },
                Measure {
                    type_: MeasureType::Max,
                    column: Some(Column {
                        human_name: "Notice Publish Start Date".to_string(),
                        pg_name: "notice_publish_start_date".to_string(),
                        data_type: DataType::CalendarDate
                    })
                },
                Measure {
                    type_: MeasureType::Min,
                    column: Some(Column {
                        human_name: "Notice Publish End Date".to_string(),
                        pg_name: "notice_publish_end_date".to_string(),
                        data_type: DataType::CalendarDate
                    })
                },
                Measure {
                    type_: MeasureType::Max,
                    column: Some(Column {
                        human_name: "Notice Publish End Date".to_string(),
                        pg_name: "notice_publish_end_date".to_string(),
                        data_type: DataType::CalendarDate
                    })
                },
                Measure {
                    type_: MeasureType::Min,
                    column: Some(Column {
                        human_name: "Notification Start Date".to_string(),
                        pg_name: "notification_start_date".to_string(),
                        data_type: DataType::CalendarDate
                    })
                },
                Measure {
                    type_: MeasureType::Max,
                    column: Some(Column {
                        human_name: "Notification Start Date".to_string(),
                        pg_name: "notification_start_date".to_string(),
                        data_type: DataType::CalendarDate
                    })
                },
                Measure {
                    type_: MeasureType::Min,
                    column: Some(Column {
                        human_name: "Notification End Date".to_string(),
                        pg_name: "notification_end_date".to_string(),
                        data_type: DataType::CalendarDate
                    })
                },
                Measure {
                    type_: MeasureType::Max,
                    column: Some(Column {
                        human_name: "Notification End Date".to_string(),
                        pg_name: "notification_end_date".to_string(),
                        data_type: DataType::CalendarDate
                    })
                },
                Measure {
                    type_: MeasureType::Min,
                    column: Some(Column {
                        human_name: "Last Updated".to_string(),
                        pg_name: "last_updated".to_string(),
                        data_type: DataType::CalendarDate
                    })
                },
                Measure {
                    type_: MeasureType::Max,
                    column: Some(Column {
                        human_name: "Last Updated".to_string(),
                        pg_name: "last_updated".to_string(),
                        data_type: DataType::CalendarDate
                    })
                },
                Measure {
                    type_: MeasureType::Min,
                    column: Some(Column {
                        human_name: "Latitude".to_string(),
                        pg_name: "latitude".to_string(),
                        data_type: DataType::Number
                    })
                },
                Measure {
                    type_: MeasureType::Max,
                    column: Some(Column {
                        human_name: "Latitude".to_string(),
                        pg_name: "latitude".to_string(),
                        data_type: DataType::Number
                    })
                },
                Measure {
                    type_: MeasureType::Sum,
                    column: Some(Column {
                        human_name: "Latitude".to_string(),
                        pg_name: "latitude".to_string(),
                        data_type: DataType::Number
                    })
                },
                Measure {
                    type_: MeasureType::Average,
                    column: Some(Column {
                        human_name: "Latitude".to_string(),
                        pg_name: "latitude".to_string(),
                        data_type: DataType::Number
                    })
                },
                Measure {
                    type_: MeasureType::Min,
                    column: Some(Column {
                        human_name: "Longitude".to_string(),
                        pg_name: "longitude".to_string(),
                        data_type: DataType::Number
                    })
                },
                Measure {
                    type_: MeasureType::Max,
                    column: Some(Column {
                        human_name: "Longitude".to_string(),
                        pg_name: "longitude".to_string(),
                        data_type: DataType::Number
                    })
                },
                Measure {
                    type_: MeasureType::Sum,
                    column: Some(Column {
                        human_name: "Longitude".to_string(),
                        pg_name: "longitude".to_string(),
                        data_type: DataType::Number
                    })
                },
                Measure {
                    type_: MeasureType::Average,
                    column: Some(Column {
                        human_name: "Longitude".to_string(),
                        pg_name: "longitude".to_string(),
                        data_type: DataType::Number
                    })
                }
            ],
        );
    }

    #[test]
    fn test_build_dimensions() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/test/socrata_resource.json");
        let resource_json = fs::read_to_string(d).unwrap();
        let resource: RawDatasetResource = serde_json::from_str(&resource_json).unwrap();
        let dataset = parse_dataset("data.edmonton.ca", &resource);

        assert_eq!(
            build_dimensions(&dataset),
            vec![
                Dimension {
                    column: Column {
                        human_name: "Row ID".to_string(),
                        pg_name: "row_id".to_string(),
                        data_type: DataType::Text
                    }
                },
                Dimension {
                    column: Column {
                        human_name: "Facility/Service Type".to_string(),
                        pg_name: "facility_service_type".to_string(),
                        data_type: DataType::Text
                    }
                },
                Dimension {
                    column: Column {
                        human_name: "Facility/Service Name".to_string(),
                        pg_name: "facility_service_name".to_string(),
                        data_type: DataType::Text
                    }
                },
                Dimension {
                    column: Column {
                        human_name: "Facility/Service Short Name".to_string(),
                        pg_name: "facility_service_short_name".to_string(),
                        data_type: DataType::Text
                    }
                },
                Dimension {
                    column: Column {
                        human_name: "Facility/Service Information".to_string(),
                        pg_name: "facility_service_information".to_string(),
                        data_type: DataType::Text
                    }
                },
                Dimension {
                    column: Column {
                        human_name: "Address".to_string(),
                        pg_name: "address".to_string(),
                        data_type: DataType::Text
                    }
                },
                Dimension {
                    column: Column {
                        human_name: "edmonton.ca Info Page".to_string(),
                        pg_name: "edmonton_ca_info_page".to_string(),
                        data_type: DataType::Url
                    }
                },
                Dimension {
                    column: Column {
                        human_name: "Notice Date Information".to_string(),
                        pg_name: "notice_date_information".to_string(),
                        data_type: DataType::Text
                    }
                },
                Dimension {
                    column: Column {
                        human_name: "Is a Service Notification".to_string(),
                        pg_name: "is_a_service_notification".to_string(),
                        data_type: DataType::Text
                    }
                },
                Dimension {
                    column: Column {
                        human_name: "Notice Start Time of Day".to_string(),
                        pg_name: "notice_start_time_of_day".to_string(),
                        data_type: DataType::Text
                    }
                },
                Dimension {
                    column: Column {
                        human_name: "Notice End Time of Day".to_string(),
                        pg_name: "notice_end_time_of_day".to_string(),
                        data_type: DataType::Text
                    }
                },
                Dimension {
                    column: Column {
                        human_name: "Notice Has Holiday Hours".to_string(),
                        pg_name: "notice_has_holiday_hours".to_string(),
                        data_type: DataType::Text
                    }
                },
                Dimension {
                    column: Column {
                        human_name: "Additional Information".to_string(),
                        pg_name: "additional_information".to_string(),
                        data_type: DataType::Text
                    }
                },
                Dimension {
                    column: Column {
                        human_name: "Status".to_string(),
                        pg_name: "status".to_string(),
                        data_type: DataType::Text
                    }
                },
                Dimension {
                    column: Column {
                        human_name: "Date Time".to_string(),
                        pg_name: "date_time".to_string(),
                        data_type: DataType::Text
                    }
                },
                Dimension {
                    column: Column {
                        human_name: "Service".to_string(),
                        pg_name: "service".to_string(),
                        data_type: DataType::Text
                    }
                },
                Dimension {
                    column: Column {
                        human_name: "Reason".to_string(),
                        pg_name: "reason".to_string(),
                        data_type: DataType::Text
                    }
                },
                Dimension {
                    column: Column {
                        human_name: "Location (address)".to_string(),
                        pg_name: "location_address".to_string(),
                        data_type: DataType::Text
                    }
                },
                Dimension {
                    column: Column {
                        human_name: "Location (city)".to_string(),
                        pg_name: "location_city".to_string(),
                        data_type: DataType::Text
                    }
                },
                Dimension {
                    column: Column {
                        human_name: "Location (state)".to_string(),
                        pg_name: "location_state".to_string(),
                        data_type: DataType::Text
                    }
                },
                Dimension {
                    column: Column {
                        human_name: "Location (zip)".to_string(),
                        pg_name: "location_zip".to_string(),
                        data_type: DataType::Text
                    }
                }
            ]
        );
    }

    #[test]
    fn test_emit_query() {
        let dataset = Dataset {
            domain: "data.edmonton.ca".to_string(),
            socrata_id: "u7r4-acwa".to_string(),
            name: "Facility/Service Notification History".to_string(),
            // Doesn't matter in this context
            columns: vec![],
        };

        let query = Query::new(
            dataset,
            vec![
                Measure {
                    type_: MeasureType::Count,
                    column: None,
                },
                Measure {
                    type_: MeasureType::Min,
                    column: Some(Column {
                        human_name: "Notification Start Date".to_string(),
                        pg_name: "notification_start_date".to_string(),
                        data_type: DataType::CalendarDate,
                    }),
                },
                Measure {
                    type_: MeasureType::Max,
                    column: Some(Column {
                        human_name: "Notification End Date".to_string(),
                        pg_name: "notification_end_date".to_string(),
                        data_type: DataType::CalendarDate,
                    }),
                },
                Measure {
                    type_: MeasureType::Average,
                    column: Some(Column {
                        human_name: "Latitude".to_string(),
                        pg_name: "latitude".to_string(),
                        data_type: DataType::Number,
                    }),
                },
                Measure {
                    type_: MeasureType::Average,
                    column: Some(Column {
                        human_name: "Longitude".to_string(),
                        pg_name: "longitude".to_string(),
                        data_type: DataType::Number,
                    }),
                },
            ],
            vec![
                Dimension {
                    column: Column {
                        human_name: "Facility/Service Type".to_string(),
                        pg_name: "facility_service_type".to_string(),
                        data_type: DataType::Text,
                    },
                },
                Dimension {
                    column: Column {
                        human_name: "Status".to_string(),
                        pg_name: "status".to_string(),
                        data_type: DataType::Text,
                    },
                },
            ],
            vec![
                OrderBy {
                    item: OrderByItem::Dimension(Dimension {
                        column: Column {
                            human_name: "Facility/Service Type".to_string(),
                            pg_name: "facility_service_type".to_string(),
                            data_type: DataType::Text,
                        },
                    }),

                    direction: OrderByDirection::Asc,
                },
                OrderBy {
                    item: OrderByItem::Measure(Measure {
                        type_: MeasureType::Min,
                        column: Some(Column {
                            human_name: "Notification Start Date".to_string(),
                            pg_name: "notification_start_date".to_string(),
                            data_type: DataType::CalendarDate,
                        }),
                    }),
                    direction: OrderByDirection::Desc,
                },
            ],
            Some(100),
            DefaultSyntax {},
        );

        assert_eq!(
            query.to_sql(),
            r#"SELECT
  facility_service_type,
  status,
  COUNT(*),
  MIN(notification_start_date) AS min_notification_start_date,
  MAX(notification_end_date) AS max_notification_end_date,
  AVG(latitude) AS avg_latitude,
  AVG(longitude) AS avg_longitude
FROM u7r4-acwa
GROUP BY
  facility_service_type,
  status
ORDER BY
  facility_service_type ASC,
  MIN(notification_start_date) DESC
LIMIT 100"#
                .to_string()
        );
    }

    #[test]
    fn test_random_query() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/test/socrata_resource.json");
        let resource_json = fs::read_to_string(d).unwrap();
        let resource: RawDatasetResource = serde_json::from_str(&resource_json).unwrap();
        let dataset = parse_dataset("data.edmonton.ca", &resource);

        let measures = build_measures(&dataset);
        let dimensions = build_dimensions(&dataset);

        // Smoke test the random query generation
        for _ in 1..10 {
            let query = random_query(
                &dataset,
                &measures,
                &dimensions,
                1..3,
                1..4,
                0..3,
                DefaultSyntax {},
            );
            let _ = query.to_sql();
        }
    }
}
