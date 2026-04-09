use chrono::{DateTime, Utc};

use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct PeriodWindow {
    pub start_utc: DateTime<Utc>,
    pub end_utc: DateTime<Utc>,
}

pub fn parse_custom_window(from_raw: &str, to_raw: &str) -> Result<PeriodWindow, AppError> {
    let from = DateTime::parse_from_rfc3339(from_raw)
        .map_err(|_| AppError::BadRequest("invalid from, expected RFC3339 datetime".to_string()))?
        .with_timezone(&Utc);
    let to = DateTime::parse_from_rfc3339(to_raw)
        .map_err(|_| AppError::BadRequest("invalid to, expected RFC3339 datetime".to_string()))?
        .with_timezone(&Utc);

    if from >= to {
        return Err(AppError::BadRequest(
            "invalid range: from must be earlier than to".to_string(),
        ));
    }

    Ok(PeriodWindow {
        start_utc: from,
        end_utc: to,
    })
}
