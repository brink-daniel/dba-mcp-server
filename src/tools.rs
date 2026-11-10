mod admin;
mod dmv;
mod querystore;
mod sqlagent;

use std::{future::Future, time::Instant};

use rmcp::{ServerHandler, handler::server::router::tool::ToolRouter, tool_handler};

pub(super) async fn log_tool_call<F>(name: &'static str, operation: F) -> String
where
    F: Future<Output = String>,
{
    let started = Instant::now();
    println!("[{}] --> MCP tool {name}", chrono::Utc::now().to_rfc3339());
    let result = operation.await;
    println!(
        "[{}] <-- MCP tool {name} ({:?})",
        chrono::Utc::now().to_rfc3339(),
        started.elapsed()
    );
    result
}

#[derive(Clone)]
pub(crate) struct SqlServerTools {
    tool_router: ToolRouter<Self>,
}

impl SqlServerTools {
    pub(crate) fn new() -> Self {
        Self {
            tool_router: SqlServerTools::timeouts_router()
                + SqlServerTools::activity_router()
                + SqlServerTools::jobs_router()
                + SqlServerTools::admin_router(),
        }
    }
}

#[tool_handler(
    router = self.tool_router,
    instructions = "This server provides strictly read-only access to records. Do not attempt modification workflows."
)]
impl ServerHandler for SqlServerTools {}
