use chrono::{DateTime, Utc};
use sqlx::Error as SqlxError;

#[derive(Debug, Clone)]
pub struct Rfc3339DateTime(pub DateTime<Utc>);

impl TryFrom<String> for Rfc3339DateTime {
    type Error = SqlxError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        DateTime::parse_from_rfc3339(&value)
            .map(|dt| Rfc3339DateTime(dt.with_timezone(&Utc)))
            .map_err(|e| SqlxError::Decode(Box::new(e)))
    }
}

impl From<Rfc3339DateTime> for DateTime<Utc> {
    fn from(value: Rfc3339DateTime) -> Self {
        value.0
    }
}

#[derive(Debug, Clone)]
pub struct OptionalRfc3339DateTime(pub Option<DateTime<Utc>>);

impl TryFrom<Option<String>> for OptionalRfc3339DateTime {
    type Error = SqlxError;

    fn try_from(value: Option<String>) -> Result<Self, Self::Error> {
        match value {
            Some(s) => DateTime::parse_from_rfc3339(&s)
                .map(|dt| OptionalRfc3339DateTime(Some(dt.with_timezone(&Utc))))
                .map_err(|e| SqlxError::Decode(Box::new(e))),
            None => Ok(OptionalRfc3339DateTime(None)),
        }
    }
}

impl From<OptionalRfc3339DateTime> for Option<DateTime<Utc>> {
    fn from(value: OptionalRfc3339DateTime) -> Self {
        value.0
    }
}
