use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Condition {
    All {
        conditions: Vec<Condition>,
    },
    Any {
        conditions: Vec<Condition>,
    },
    Not {
        condition: Box<Condition>,
    },
    TimeWindow {
        timezone: String,
        days_of_week: Vec<String>,
        start: String,
        end: String,
    },
    DateRange {
        start: NaiveDate,
        end: NaiveDate,
        timezone: String,
    },
    DeviceType {
        r#in: Vec<Device>,
    },
    QueryParameterEquals {
        name: String,
        value: String,
    },
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Device {
    Mobile,
    Tablet,
    Desktop,
    Bot,
    Unknown,
}
