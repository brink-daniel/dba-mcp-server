use rmcp::{handler::server::wrapper::Parameters, schemars, tool, tool_router};
use serde::Deserialize;

use super::{SqlServerTools, log_tool_call};
use crate::query_instance;

const CURRENT_ACTIVITY_QUERY: &str = include_str!("../../queries/current_activity.sql");
const DATABASES_BY_INSTANCE_QUERY: &str = include_str!("../../queries/databases_by_instance.sql");
const DISK_SPACE_QUERY: &str = include_str!("../../queries/disk_space.sql");
const DATABASE_FILE_USAGE_QUERY: &str = include_str!("../../queries/database_file_usage.sql");
const WAIT_STATS_QUERY: &str = include_str!("../../queries/wait_stats.sql");
const TOP_QUERIES_BY_CPU_QUERY: &str = include_str!("../../queries/top_queries_by_cpu.sql");
const MISSING_INDEXES_QUERY: &str = include_str!("../../queries/missing_indexes.sql");
const TEMPDB_USAGE_QUERY: &str = include_str!("../../queries/tempdb_usage.sql");

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct InstanceArgs {
    /// SQL Server instance to query, for example localhost or localhost:1433.
    instance: String,
}

#[tool_router(router = activity_router, vis = "pub(super)")]
impl SqlServerTools {
    #[tool(
        name = "get_current_activity",
        description = "Retrieves the current SQL Server activity",
        annotations(read_only_hint = true, idempotent_hint = true)
    )]
    async fn get_current_activity(&self, Parameters(args): Parameters<InstanceArgs>) -> String {
        log_tool_call("get_current_activity", async {
            query_instance(&args.instance, None, CURRENT_ACTIVITY_QUERY.trim(), &[])
                .await
                .unwrap_or_else(|error| format!("SQL Server error: {error:#}"))
        })
        .await
    }

    #[tool(
        name = "get_databases",
        description = "Lists the databases available on a SQL Server instance",
        annotations(read_only_hint = true, idempotent_hint = true)
    )]
    async fn get_databases(&self, Parameters(args): Parameters<InstanceArgs>) -> String {
        log_tool_call("get_databases", async {
            query_instance(
                &args.instance,
                None,
                DATABASES_BY_INSTANCE_QUERY.trim(),
                &[],
            )
            .await
            .unwrap_or_else(|error| format!("SQL Server error: {error:#}"))
        })
        .await
    }

    #[tool(
        name = "get_disk_space",
        description = "Reports total and available disk space for volumes containing SQL Server database files",
        annotations(read_only_hint = true, idempotent_hint = true)
    )]
    async fn get_disk_space(&self, Parameters(args): Parameters<InstanceArgs>) -> String {
        log_tool_call("get_disk_space", async {
            query_instance(&args.instance, None, DISK_SPACE_QUERY.trim(), &[])
                .await
                .unwrap_or_else(|error| format!("SQL Server error: {error:#}"))
        })
        .await
    }

    #[tool(
        name = "get_database_file_usage",
        description = "Reports allocated size, maximum size, and autogrowth settings for database files",
        annotations(read_only_hint = true, idempotent_hint = true)
    )]
    async fn get_database_file_usage(&self, Parameters(args): Parameters<InstanceArgs>) -> String {
        log_tool_call("get_database_file_usage", async {
            query_instance(&args.instance, None, DATABASE_FILE_USAGE_QUERY.trim(), &[])
                .await
                .unwrap_or_else(|error| format!("SQL Server error: {error:#}"))
        })
        .await
    }

    #[tool(
        name = "get_wait_stats",
        description = "Reports cumulative SQL Server wait statistics since the last SQL Server restart; interpret waits in the context of workload and uptime",
        annotations(read_only_hint = true, idempotent_hint = true)
    )]
    async fn get_wait_stats(&self, Parameters(args): Parameters<InstanceArgs>) -> String {
        log_tool_call("get_wait_stats", async {
            query_instance(&args.instance, None, WAIT_STATS_QUERY.trim(), &[])
                .await
                .unwrap_or_else(|error| format!("SQL Server error: {error:#}"))
        })
        .await
    }

    #[tool(
        name = "get_top_queries_by_cpu",
        description = "Shows the top 25 cached SQL statements by cumulative CPU since their plans were compiled; this is plan-cache data, not a time-bounded workload report",
        annotations(read_only_hint = true, idempotent_hint = true)
    )]
    async fn get_top_queries_by_cpu(&self, Parameters(args): Parameters<InstanceArgs>) -> String {
        log_tool_call("get_top_queries_by_cpu", async {
            query_instance(&args.instance, None, TOP_QUERIES_BY_CPU_QUERY.trim(), &[])
                .await
                .unwrap_or_else(|error| format!("SQL Server error: {error:#}"))
        })
        .await
    }

    #[tool(
        name = "get_missing_indexes",
        description = "Lists missing-index DMV suggestions ranked by estimated impact; suggestions are advisory and should be reviewed against existing indexes and workload",
        annotations(read_only_hint = true, idempotent_hint = true)
    )]
    async fn get_missing_indexes(&self, Parameters(args): Parameters<InstanceArgs>) -> String {
        log_tool_call("get_missing_indexes", async {
            query_instance(&args.instance, None, MISSING_INDEXES_QUERY.trim(), &[])
                .await
                .unwrap_or_else(|error| format!("SQL Server error: {error:#}"))
        })
        .await
    }

    #[tool(
        name = "get_tempdb_usage",
        description = "Reports current tempdb file space and sessions with the largest allocated or deallocated tempdb space",
        annotations(read_only_hint = true, idempotent_hint = true)
    )]
    async fn get_tempdb_usage(&self, Parameters(args): Parameters<InstanceArgs>) -> String {
        log_tool_call("get_tempdb_usage", async {
            query_instance(
                &args.instance,
                Some("tempdb"),
                TEMPDB_USAGE_QUERY.trim(),
                &[],
            )
            .await
            .unwrap_or_else(|error| format!("SQL Server error: {error:#}"))
        })
        .await
    }
}
