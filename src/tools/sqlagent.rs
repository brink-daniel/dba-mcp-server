use chrono::NaiveDateTime;
use rmcp::{handler::server::wrapper::Parameters, schemars, tool, tool_router};
use serde::Deserialize;
use tiberius::ToSql;

use super::{SqlServerTools, log_tool_call};
use crate::query_instance;

const JOBS_BY_INSTANCE_QUERY: &str = include_str!("../../queries/jobs_by_instance.sql");
const JOB_HISTORY_QUERY: &str = include_str!("../../queries/job_history_by_job_name.sql");
const JOBS_RUNNING_QUERY: &str =
    include_str!("../../queries/jobs_running_between_start_end_time.sql");

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct InstanceArgs {
    /// SQL Server instance to query, for example localhost or localhost:1433.
    instance: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct JobHistoryArgs {
    /// SQL Server instance to query, for example localhost or localhost:1433.
    instance: String,
    /// SQL Server Agent job name.
    job_name: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct JobsRunningBetweenArgs {
    /// SQL Server instance to query, for example localhost or localhost:1433.
    instance: String,
    /// Start time in ISO 8601 format.
    start_datetime: String,
    /// End time in ISO 8601 format.
    end_datetime: String,
}

#[tool_router(router = jobs_router, vis = "pub(super)")]
#[allow(non_snake_case)]
impl SqlServerTools {
    #[tool(description = "Lists SQL Server Agent jobs on a SQL Server instance")]
    async fn get_sqlagent_jobs(&self, Parameters(args): Parameters<InstanceArgs>) -> String {
        log_tool_call("get_sqlagent_jobs", async {
            query_instance(
                &args.instance,
                Some("msdb"),
                JOBS_BY_INSTANCE_QUERY.trim(),
                &[],
            )
            .await
            .unwrap_or_else(|error| format!("SQL Server error: {error:#}"))
        })
        .await
    }

    #[tool(description = "Retrieves execution history for a SQL Server Agent job by name")]
    async fn get_sqlagent_job_history_by_job_name(
        &self,
        Parameters(args): Parameters<JobHistoryArgs>,
    ) -> String {
        log_tool_call("get_sqlagent_job_history_by_job_name", async {
            let params: [&dyn ToSql; 1] = [&args.job_name];
            query_instance(
                &args.instance,
                Some("msdb"),
                JOB_HISTORY_QUERY.trim(),
                &params,
            )
            .await
            .unwrap_or_else(|error| format!("SQL Server error: {error:#}"))
        })
        .await
    }

    #[tool(description = "Lists SQL Server Agent job executions that overlapped a time range")]
    async fn get_sqlagent_jobs_running_between_start_end_time(
        &self,
        Parameters(args): Parameters<JobsRunningBetweenArgs>,
    ) -> String {
        log_tool_call("get_sqlagent_jobs_running_between_start_end_time", async {
            let parse = |value: &str| NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S%.f");
            let start = match parse(&args.start_datetime) {
                Ok(value) => value,
                Err(error) => return format!("Invalid start_datetime: {error}"),
            };
            let end = match parse(&args.end_datetime) {
                Ok(value) => value,
                Err(error) => return format!("Invalid end_datetime: {error}"),
            };
            if start > end {
                return "Invalid time range: start_datetime must be before or equal to end_datetime"
                    .into();
            }
            let params: [&dyn ToSql; 2] = [&start, &end];
            query_instance(
                &args.instance,
                Some("msdb"),
                JOBS_RUNNING_QUERY.trim(),
                &params,
            )
            .await
            .unwrap_or_else(|error| format!("SQL Server error: {error:#}"))
        })
        .await
    }
}
