mod admin;
mod dmv;
mod querystore;
mod sqlagent;

use rmcp::{ServerHandler, handler::server::router::tool::ToolRouter, tool_handler};

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

#[tool_handler(router = self.tool_router)]
impl ServerHandler for SqlServerTools {}
