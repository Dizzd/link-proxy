use anyhow::{Result, bail, ensure};
use chrono::{DateTime, Datelike, NaiveDate, NaiveTime, Utc};
use chrono_tz::Tz;
use proxy_domain::{Condition, Device};

pub trait Clock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}
pub struct SystemClock;
impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}
pub struct FixedClock(pub DateTime<Utc>);
impl Clock for FixedClock {
    fn now(&self) -> DateTime<Utc> {
        self.0
    }
}

pub struct RequestContext<'a> {
    pub now: DateTime<Utc>,
    pub user_agent: &'a str,
    pub query: &'a str,
}

#[derive(Clone, Debug)]
pub enum CompiledCondition {
    All(Vec<Self>),
    Any(Vec<Self>),
    Not(Box<Self>),
    Time {
        timezone: Tz,
        days: Vec<u32>,
        start: NaiveTime,
        end: NaiveTime,
    },
    Date {
        timezone: Tz,
        start: NaiveDate,
        end: NaiveDate,
    },
    Device(Vec<Device>),
    Query {
        name: String,
        value: String,
    },
}
impl CompiledCondition {
    pub fn compile(condition: &Condition) -> Result<Self> {
        let mut nodes = 0;
        Self::compile_inner(condition, 0, &mut nodes)
    }
    fn compile_inner(c: &Condition, depth: usize, nodes: &mut usize) -> Result<Self> {
        *nodes += 1;
        ensure!(
            depth <= 12 && *nodes <= 128,
            "condition exceeds depth 12 or 128 nodes"
        );
        let mut children = |list: &[Condition]| -> Result<Vec<Self>> {
            ensure!(!list.is_empty(), "boolean group must not be empty");
            list.iter()
                .map(|c| Self::compile_inner(c, depth + 1, nodes))
                .collect()
        };
        Ok(match c {
            Condition::All { conditions } => Self::All(children(conditions)?),
            Condition::Any { conditions } => Self::Any(children(conditions)?),
            Condition::Not { condition } => {
                Self::Not(Box::new(Self::compile_inner(condition, depth + 1, nodes)?))
            }
            Condition::TimeWindow {
                timezone,
                days_of_week,
                start,
                end,
            } => {
                ensure!(!days_of_week.is_empty(), "weekdays required");
                let days = days_of_week
                    .iter()
                    .map(|s| match s.as_str() {
                        "mon" => Ok(0),
                        "tue" => Ok(1),
                        "wed" => Ok(2),
                        "thu" => Ok(3),
                        "fri" => Ok(4),
                        "sat" => Ok(5),
                        "sun" => Ok(6),
                        _ => bail!("invalid weekday: {s}"),
                    })
                    .collect::<Result<Vec<_>>>()?;
                let start = NaiveTime::parse_from_str(start, "%H:%M")?;
                let end = NaiveTime::parse_from_str(end, "%H:%M")?;
                ensure!(start != end, "time window cannot have equal boundaries");
                Self::Time {
                    timezone: timezone.parse()?,
                    days,
                    start,
                    end,
                }
            }
            Condition::DateRange {
                start,
                end,
                timezone,
            } => {
                ensure!(start <= end, "date range is reversed");
                Self::Date {
                    timezone: timezone.parse()?,
                    start: *start,
                    end: *end,
                }
            }
            Condition::DeviceType { r#in } => {
                ensure!(!r#in.is_empty(), "device list required");
                Self::Device(r#in.clone())
            }
            Condition::QueryParameterEquals { name, value } => {
                ensure!(
                    !name.is_empty() && name.len() <= 256 && value.len() <= 2048,
                    "invalid query condition length"
                );
                Self::Query {
                    name: name.clone(),
                    value: value.clone(),
                }
            }
        })
    }
    pub fn matches(&self, ctx: &RequestContext<'_>) -> bool {
        match self {
            Self::All(cs) => cs.iter().all(|c| c.matches(ctx)),
            Self::Any(cs) => cs.iter().any(|c| c.matches(ctx)),
            Self::Not(c) => !c.matches(ctx),
            Self::Time {
                timezone,
                days,
                start,
                end,
            } => {
                let local = ctx.now.with_timezone(timezone);
                let time = local.time();
                // Overnight windows belong to the weekday on which the window starts.
                let day = if start > end && time < *end {
                    local
                        .date_naive()
                        .pred_opt()
                        .map_or(local.weekday(), |d| d.weekday())
                } else {
                    local.weekday()
                };
                days.contains(&day.num_days_from_monday())
                    && if start < end {
                        time >= *start && time < *end
                    } else {
                        time >= *start || time < *end
                    }
            }
            Self::Date {
                timezone,
                start,
                end,
            } => {
                let date = ctx.now.with_timezone(timezone).date_naive();
                date >= *start && date <= *end
            }
            Self::Device(devices) => devices.contains(&detect_device(ctx.user_agent)),
            Self::Query { name, value } => url::form_urlencoded::parse(ctx.query.as_bytes())
                .any(|(k, v)| k == name.as_str() && v == value.as_str()),
        }
    }
}

pub fn detect_device(user_agent: &str) -> Device {
    // Bounded input, heuristic classification only; never used for authorization.
    let ua: String = user_agent
        .chars()
        .take(2048)
        .flat_map(char::to_lowercase)
        .collect();
    if ua.contains("bot") || ua.contains("spider") || ua.contains("crawler") {
        Device::Bot
    } else if ua.contains("ipad")
        || ua.contains("tablet")
        || (ua.contains("android") && !ua.contains("mobile"))
    {
        Device::Tablet
    } else if ua.contains("mobile") || ua.contains("iphone") {
        Device::Mobile
    } else if ua.contains("windows") || ua.contains("macintosh") || ua.contains("x11") {
        Device::Desktop
    } else {
        Device::Unknown
    }
}
