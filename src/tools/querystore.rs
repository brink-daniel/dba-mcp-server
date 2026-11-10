use rmcp::{handler::server::wrapper::Parameters, schemars, tool, tool_router};
use serde::Deserialize;
use tiberius::ToSql;

use super::{SqlServerTools, log_tool_call};
use crate::query_instance;

const ABORTED_QUERY: &str = include_str!("../../queries/query_execution_aborted.sql");
const EXCEPTION_QUERY: &str = include_str!("../../queries/query_execution_exception.sql");

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct QueryExecutionArgs {
    /// SQL Server instance to query, for example localhost or localhost:1433.
    instance: String,
    /// Database to query.
    database: String,
    /// Start time in ISO 8601 format. Defaults to one hour ago.
    start_datetime: Option<String>,
    /// End time in ISO 8601 format. Defaults to now.
    end_datetime: Option<String>,
}

#[tool_router(router = query_store_router, vis = "pub(super)")]
impl SqlServerTools {
    #[tool(
        name = "get_query_execution_aborted",
        description = "Retrieves client-aborted Query Store execution aggregates in intervals overlapping the given time range",
        annotations(read_only_hint = true, idempotent_hint = true)
    )]
    async fn get_query_execution_aborted(
        &self,
        Parameters(args): Parameters<QueryExecutionArgs>,
    ) -> String {
        log_tool_call("get_query_execution_aborted", async {
            run_query(&args, ABORTED_QUERY).await
        })
        .await
    }

    #[tool(
        name = "get_query_execution_exception",
        description = "Retrieves Query Store execution exception aggregates in intervals overlapping the given time range",
        annotations(read_only_hint = true, idempotent_hint = true)
    )]
    async fn get_query_execution_exception(
        &self,
        Parameters(args): Parameters<QueryExecutionArgs>,
    ) -> String {
        log_tool_call("get_query_execution_exception", async {
            run_query(&args, EXCEPTION_QUERY).await
        })
        .await
    }
}

async fn run_query(args: &QueryExecutionArgs, query: &str) -> String {
    let now = chrono::Local::now().naive_local();
    let start = match parse_datetime(args.start_datetime.as_deref(), now - chrono::Duration::hours(1)) {
        Ok(value) => value,
        Err(message) => return format!("Invalid start_datetime: {message}"),
    };
    let end = match parse_datetime(args.end_datetime.as_deref(), now) {
        Ok(value) => value,
        Err(message) => return format!("Invalid end_datetime: {message}"),
    };
    if start > end {
        return "Invalid time range: start_datetime must be before or equal to end_datetime".into();
    }
    let params: [&dyn ToSql; 2] = [&start, &end];
    query_instance(&args.instance, Some(&args.database), query.trim(), &params)
        .await
        .unwrap_or_else(|error| format!("SQL Server error: {error:#}"))
}

fn parse_datetime(value: Option<&str>, default: chrono::NaiveDateTime) -> Result<chrono::NaiveDateTime, String> {
    match value {
        Some(value) => chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S%.f")
            .map_err(|error| error.to_string()),
        None => Ok(default),
    }
}
