use rmcp::{handler::server::wrapper::Parameters, schemars, tool, tool_router};
use serde::Deserialize;
use tiberius::ToSql;

use super::SqlServerTools;
use crate::query_instance;

const BACKUP_HISTORY_QUERY: &str = include_str!("../../queries/backup_history_by_database.sql");
const RESTORE_HISTORY_QUERY: &str = include_str!("../../queries/restore_history_by_database.sql");
const SERVER_ERRORLOG_QUERY: &str = include_str!("../../queries/server_errorlog.sql");
const AGENT_ERRORLOG_QUERY: &str = include_str!("../../queries/agent_errorlog.sql");
const AVAILABILITY_GROUP_STATUS_QUERY: &str =
    include_str!("../../queries/availability_group_status.sql");

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct BackupHistoryArgs {
    /// SQL Server instance to query, for example localhost or localhost:1433.
    instance: String,
    /// Database whose history should be retrieved.
    database: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ErrorLogArgs {
    /// SQL Server instance to query, for example localhost or localhost:1433.
    instance: String,
    /// Error log number: 0 is current, 1 is previous, and so on. Defaults to 0.
    log_number: Option<i32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct AvailabilityGroupStatusArgs {
    /// SQL Server instance to query, for example localhost or localhost:1433.
    instance: String,
}

#[tool_router(router = admin_router, vis = "pub(super)")]
impl SqlServerTools {
    #[tool(description = "Retrieves backup history for a database on a SQL Server instance")]
    async fn get_backup_history_by_database(
        &self,
        Parameters(args): Parameters<BackupHistoryArgs>,
    ) -> String {
        let params: [&dyn ToSql; 1] = [&args.database];
        query_instance(
            &args.instance,
            Some("msdb"),
            BACKUP_HISTORY_QUERY.trim(),
            &params,
        )
        .await
        .unwrap_or_else(|error| format!("SQL Server error: {error:#}"))
    }

    #[tool(description = "Retrieves restore history for a database on a SQL Server instance")]
    async fn get_restore_history_by_database(
        &self,
        Parameters(args): Parameters<BackupHistoryArgs>,
    ) -> String {
        let params: [&dyn ToSql; 1] = [&args.database];
        query_instance(
            &args.instance,
            Some("msdb"),
            RESTORE_HISTORY_QUERY.trim(),
            &params,
        )
        .await
        .unwrap_or_else(|error| format!("SQL Server error: {error:#}"))
    }

    #[tool(description = "Reads entries from the SQL Server error log")]
    async fn get_server_errorlog(&self, Parameters(args): Parameters<ErrorLogArgs>) -> String {
        read_error_log(
            &args.instance,
            args.log_number.unwrap_or(0),
            SERVER_ERRORLOG_QUERY,
        )
        .await
    }

    #[tool(description = "Reads entries from the SQL Server Agent error log")]
    async fn get_agent_errorlog(&self, Parameters(args): Parameters<ErrorLogArgs>) -> String {
        read_error_log(
            &args.instance,
            args.log_number.unwrap_or(0),
            AGENT_ERRORLOG_QUERY,
        )
        .await
    }

    #[tool(
        description = "Retrieves availability group and replica health status for a SQL Server instance"
    )]
    async fn get_availability_group_status(
        &self,
        Parameters(args): Parameters<AvailabilityGroupStatusArgs>,
    ) -> String {
        query_instance(
            &args.instance,
            None,
            AVAILABILITY_GROUP_STATUS_QUERY.trim(),
            &[],
        )
        .await
        .unwrap_or_else(|error| format!("SQL Server error: {error:#}"))
    }
}

async fn read_error_log(instance: &str, log_number: i32, query: &str) -> String {
    if log_number < 0 {
        return "Invalid log_number: it must be zero or greater".into();
    }
    let params: [&dyn ToSql; 1] = [&log_number];
    query_instance(instance, None, query.trim(), &params)
        .await
        .unwrap_or_else(|error| format!("SQL Server error: {error:#}"))
}
