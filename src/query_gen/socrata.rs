use itertools::izip;
use serde_derive::Deserialize;
use serde_enum_str::Deserialize_enum_str;

#[derive(Deserialize_enum_str, PartialEq, Eq, Debug, Clone)]
pub enum DataType {
    // ~ PG bool
    Checkbox,
    Double,
    // PG timestamp
    #[serde(rename = "Floating timestamp")]
    FloatingTimestamp,
    // PG date
    #[serde(rename = "Calendar date")]
    CalendarDate,
    Money,
    Number,
    Text,
    Url,

    // JSON stuff
    Line,
    Location,
    Multiline,
    Multipoint,
    Multipolygon,
    Point,

    #[serde(other)]
    Other(String),
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Column {
    pub human_name: String,
    pub pg_name: String,
    pub data_type: DataType,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Dataset {
    pub domain: String,
    pub socrata_id: String,
    pub name: String,
    pub columns: Vec<Column>,
}

#[derive(Deserialize)]
pub struct RawDatasetResource {
    name: String,
    id: String,
    description: String,
    columns_name: Vec<String>,
    columns_field_name: Vec<String>,
    columns_datatype: Vec<DataType>,
}

pub fn parse_dataset(domain: &str, resource: &RawDatasetResource) -> Dataset {
    let mut columns: Vec<Column> = Vec::new();

    for (human_name, pg_name, data_type) in izip!(
        resource.columns_name.clone(),
        resource.columns_field_name.clone(),
        resource.columns_datatype.clone()
    ) {
        columns.push(Column {
            human_name,
            pg_name,
            data_type,
        })
    }

    Dataset {
        domain: domain.to_string(),
        socrata_id: resource.id.clone(),
        name: resource.name.clone(),
        columns,
    }
}

#[cfg(test)]
mod tests {
    use crate::query_gen::socrata::{parse_dataset, Column, DataType, Dataset, RawDatasetResource};
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn test_parse_dataset() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/test/socrata_resource.json");

        let resource_json = fs::read_to_string(d).unwrap();
        let resource: RawDatasetResource = serde_json::from_str(&resource_json).unwrap();
        let dataset = parse_dataset("data.edmonton.ca", &resource);

        assert_eq!(
            dataset,
            Dataset {
                domain: "data.edmonton.ca".to_string(),
                socrata_id: "u7r4-acwa".to_string(),
                name: "Facility/Service Notification History".to_string(),
                columns: vec![
                    Column { human_name: "Row ID".to_string(), pg_name: "row_id".to_string(), data_type: DataType::Text },
                    Column { human_name: "Facility/Service Type".to_string(), pg_name: "facility_service_type".to_string(), data_type: DataType::Text },
                    Column { human_name: "Facility/Service ID".to_string(), pg_name: "facility_service_id".to_string(), data_type: DataType::Number },
                    Column { human_name: "Facility/Service Name".to_string(), pg_name: "facility_service_name".to_string(), data_type: DataType::Text },
                    Column { human_name: "Facility/Service Short Name".to_string(), pg_name: "facility_service_short_name".to_string(), data_type: DataType::Text },
                    Column { human_name: "Facility/Service Information".to_string(), pg_name: "facility_service_information".to_string(), data_type: DataType::Text },
                    Column { human_name: "Address".to_string(), pg_name: "address".to_string(), data_type: DataType::Text },
                    Column { human_name: "edmonton.ca Info Page".to_string(), pg_name: "edmonton_ca_info_page".to_string(), data_type: DataType::Url },
                    Column { human_name: "Notification ID".to_string(), pg_name: "notification_id".to_string(), data_type: DataType::Number },
                    Column { human_name: "Notice Publish Start Date".to_string(), pg_name: "notice_publish_start_date".to_string(), data_type: DataType::CalendarDate },
                    Column { human_name: "Notice Publish End Date".to_string(), pg_name: "notice_publish_end_date".to_string(), data_type: DataType::CalendarDate },
                    Column { human_name: "Notice Date Information".to_string(), pg_name: "notice_date_information".to_string(), data_type: DataType::Text },
                    Column { human_name: "Notification Start Date".to_string(), pg_name: "notification_start_date".to_string(), data_type: DataType::CalendarDate },
                    Column { human_name: "Notification End Date".to_string(), pg_name: "notification_end_date".to_string(), data_type: DataType::CalendarDate },
                    Column { human_name: "Is a Service Notification".to_string(), pg_name: "is_a_service_notification".to_string(), data_type: DataType::Text },
                    Column { human_name: "Notice Start Time of Day".to_string(), pg_name: "notice_start_time_of_day".to_string(), data_type: DataType::Text },
                    Column { human_name: "Notice End Time of Day".to_string(), pg_name: "notice_end_time_of_day".to_string(), data_type: DataType::Text },
                    Column { human_name: "Notice Has Holiday Hours".to_string(), pg_name: "notice_has_holiday_hours".to_string(), data_type: DataType::Text },
                    Column { human_name: "Additional Information".to_string(), pg_name: "additional_information".to_string(), data_type: DataType::Text },
                    Column { human_name: "Status".to_string(), pg_name: "status".to_string(), data_type: DataType::Text },
                    Column { human_name: "Date Time".to_string(), pg_name: "date_time".to_string(), data_type: DataType::Text },
                    Column { human_name: "Service".to_string(), pg_name: "service".to_string(), data_type: DataType::Text },
                    Column { human_name: "Reason".to_string(), pg_name: "reason".to_string(), data_type: DataType::Text },
                    Column { human_name: "Last Updated".to_string(), pg_name: "last_updated".to_string(), data_type: DataType::CalendarDate },
                    Column { human_name: "Latitude".to_string(), pg_name: "latitude".to_string(), data_type: DataType::Number },
                    Column { human_name: "Longitude".to_string(), pg_name: "longitude".to_string(), data_type: DataType::Number },
                    Column { human_name: "Location".to_string(), pg_name: "location".to_string(), data_type: DataType::Point },
                    Column { human_name: "Location (address)".to_string(), pg_name: "location_address".to_string(), data_type: DataType::Text },
                    Column { human_name: "Geometry Point".to_string(), pg_name: "geometry_point".to_string(), data_type: DataType::Point },
                    Column { human_name: "Location (city)".to_string(), pg_name: "location_city".to_string(), data_type: DataType::Text },
                    Column { human_name: "Location (state)".to_string(), pg_name: "location_state".to_string(), data_type: DataType::Text },
                    Column { human_name: "Location (zip)".to_string(), pg_name: "location_zip".to_string(), data_type: DataType::Text },
                    Column { human_name: "Neighbourhood Boundaries : 2019".to_string(), pg_name: ":@computed_region_7ccj_gre3".to_string(), data_type: DataType::Number },
                    Column { human_name: "Roadway Maintenance Area Polygon".to_string(), pg_name: ":@computed_region_ecxu_fw7u".to_string(), data_type: DataType::Number },
                    Column { human_name: "Edmonton Public School Board (EPSB) Ward Boundaries (effective at 12:00 AM on Oct 16, 2017)".to_string(), pg_name: ":@computed_region_izdr_ja4x".to_string(), data_type: DataType::Number },
                    Column { human_name: "Edmonton Catholic School District Ward Boundaries (effective at 12:00 AM on Oct 16, 2017)".to_string(), pg_name: ":@computed_region_5jki_au6x".to_string(), data_type: DataType::Number },
                    Column { human_name: "City of Edmonton - Ward Boundaries (effective at 12:00 AM on Oct 16, 2017)".to_string(), pg_name: ":@computed_region_mnf4_kaez".to_string(), data_type: DataType::Number },
                    Column { human_name: "City of Edmonton : Neighbourhood Boundaries".to_string(), pg_name: ":@computed_region_eq8d_jmrp".to_string(), data_type: DataType::Number }],
            }
        )
    }
}
