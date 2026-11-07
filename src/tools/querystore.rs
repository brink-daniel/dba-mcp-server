use rmcp::{handler::server::wrapper::Parameters, schemars, tool, tool_router};
use serde::Deserialize;
use tiberius::ToSql;

use super::{SqlServerTools, log_tool_call};
use crate::query_instance;

const TIMEOUTS_QUERY: &str = include_str!("../../queries/timeouts.sql");

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct TimeoutArgs {
    /// SQL Server instance to query, for example localhost or localhost:1433.
    instance: String,
    /// Database to query for timeouts.
    database: String,
    /// Start time in ISO 8601 format. Defaults to one hour ago.
    start_datetime: Option<String>,
    /// End time in ISO 8601 format. Defaults to now.
    end_datetime: Option<String>,
}

#[tool_router(router = timeouts_router, vis = "pub(super)")]
impl SqlServerTools {
    #[tool(
        description = "Retrieves client-aborted Query Store executions in intervals overlapping the given time range; these may include timeouts"
    )]
    async fn get_timeouts(&self, Parameters(args): Parameters<TimeoutArgs>) -> String {
        log_tool_call("get_timeouts", async {
            let now = chrono::Local::now().naive_local();
            let start = match args.start_datetime {
                Some(value) => {
                    match chrono::NaiveDateTime::parse_from_str(&value, "%Y-%m-%dT%H:%M:%S%.f") {
                        Ok(value) => value,
                        Err(error) => return format!("Invalid start_datetime: {error}"),
                    }
                }
                None => now - chrono::Duration::hours(1),
            };
            let end = match args.end_datetime {
                Some(value) => {
                    match chrono::NaiveDateTime::parse_from_str(&value, "%Y-%m-%dT%H:%M:%S%.f") {
                        Ok(value) => value,
                        Err(error) => return format!("Invalid end_datetime: {error}"),
                    }
                }
                None => now,
            };
            if start > end {
                return "Invalid time range: start_datetime must be before or equal to end_datetime"
                .into();
            }
            let params: [&dyn ToSql; 2] = [&start, &end];
            query_instance(
                &args.instance,
                Some(&args.database),
                TIMEOUTS_QUERY.trim(),
                &params,
            )
            .await
            .unwrap_or_else(|error| format!("SQL Server error: {error:#}"))
        })
        .await
    }
}
