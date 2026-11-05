use rmcp::{handler::server::wrapper::Parameters, schemars, tool, tool_router};
use serde::Deserialize;

use super::SqlServerTools;
use crate::query_instance;

const CURRENT_ACTIVITY_QUERY: &str = include_str!("../../queries/current_activity.sql");
const DATABASES_BY_INSTANCE_QUERY: &str = include_str!("../../queries/databases_by_instance.sql");
const DISK_SPACE_QUERY: &str = include_str!("../../queries/disk_space.sql");
const DATABASE_FILE_USAGE_QUERY: &str = include_str!("../../queries/database_file_usage.sql");

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct InstanceArgs {
    /// SQL Server instance to query, for example localhost or localhost:1433.
    instance: String,
}

#[tool_router(router = activity_router, vis = "pub(super)")]
impl SqlServerTools {
    #[tool(description = "Retrieves the current SQL Server activity")]
    async fn get_current_activity(&self, Parameters(args): Parameters<InstanceArgs>) -> String {
        query_instance(&args.instance, None, CURRENT_ACTIVITY_QUERY.trim(), &[])
            .await
            .unwrap_or_else(|error| format!("SQL Server error: {error:#}"))
    }

    #[tool(description = "Lists the databases available on a SQL Server instance")]
    async fn get_databases(&self, Parameters(args): Parameters<InstanceArgs>) -> String {
        query_instance(
            &args.instance,
            None,
            DATABASES_BY_INSTANCE_QUERY.trim(),
            &[],
        )
        .await
        .unwrap_or_else(|error| format!("SQL Server error: {error:#}"))
    }

    #[tool(
        description = "Reports total and available disk space for volumes containing SQL Server database files"
    )]
    async fn get_disk_space(&self, Parameters(args): Parameters<InstanceArgs>) -> String {
        query_instance(&args.instance, None, DISK_SPACE_QUERY.trim(), &[])
            .await
            .unwrap_or_else(|error| format!("SQL Server error: {error:#}"))
    }

    #[tool(
        description = "Reports allocated size, maximum size, and autogrowth settings for database files"
    )]
    async fn get_database_file_usage(&self, Parameters(args): Parameters<InstanceArgs>) -> String {
        query_instance(&args.instance, None, DATABASE_FILE_USAGE_QUERY.trim(), &[])
            .await
            .unwrap_or_else(|error| format!("SQL Server error: {error:#}"))
    }
}
