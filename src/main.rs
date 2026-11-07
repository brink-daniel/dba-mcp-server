use std::{net::SocketAddr, sync::Arc};

use anyhow::{Context, Result, bail, ensure};
use axum::{
    Router,
    extract::{Request, State},
    http::{StatusCode, header::AUTHORIZATION},
    middleware::{Next, from_fn_with_state},
    response::{IntoResponse, Response},
};
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};
use tower_http::cors::{Any, CorsLayer};

async fn require_bearer_token(
    State(token): State<Arc<str>>,
    request: Request,
    next: Next,
) -> Response {
    let expected = format!("Bearer {token}");
    let provided = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok());

    if provided == Some(expected.as_str()) {
        next.run(request).await
    } else {
        (StatusCode::UNAUTHORIZED, [("www-authenticate", "Bearer")]).into_response()
    }
}

mod tools;

use tools::SqlServerTools;

mod db;
pub(crate) use db::query_instance;

// Named command-line parameters: --help, --version, --port <port>,
// --path <path>, --dns-name <name>, and --token <token>. Defaults are port
// 8080 and path /mcp; the DNS name and token are optional. If token is omitted,
// a cryptographically random token is generated at startup.
// Examples:
//   cargo run
//   cargo run -- --port 9000 --path /database-tools --dns-name dba.example.com
//   cargo run -- --token my-predefined-token
#[tokio::main]
async fn main() -> Result<()> {
    let options = parse_args()?;
    let port = options.port;
    let path = options.path;
    let dns_name = options.dns_name;
    let token_arg = options.token;
    let token = match token_arg {
        Some(token) => token,
        None => generate_token()?,
    };
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let server_name = std::env::var("COMPUTERNAME")
        .ok()
        .filter(|name| !name.trim().is_empty());
    let mut allowed_hosts = vec![
        "localhost".to_string(),
        "127.0.0.1".to_string(),
        "::1".to_string(),
    ];
    if let Some(name) = &server_name {
        allowed_hosts.push(name.clone());
    }
    if let Some(name) = &dns_name {
        allowed_hosts.push(name.clone());
    }

    let service = StreamableHttpService::new(
        || Ok(SqlServerTools::new()),
        LocalSessionManager::default().into(),
        StreamableHttpServerConfig::default().with_allowed_hosts(allowed_hosts),
    );

    let app = Router::new()
        .nest_service(&path, service)
        .layer(from_fn_with_state(
            Arc::<str>::from(token.as_str()),
            require_bearer_token,
        ))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_headers(Any)
                .allow_methods(Any),
        );

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("Could not bind MCP server to {addr}"))?;
    println!("Streamable HTTP MCP server listening on {addr}");
    println!("MCP endpoint: http://localhost:{port}{path}");
    if let Some(name) = server_name {
        println!("MCP endpoint: http://{name}:{port}{path}");
    }
    if let Some(name) = dns_name {
        println!("MCP endpoint: http://{name}:{port}{path}");
    }
    println!("Authorization: Bearer {token}");
    axum::serve(listener, app).await?;
    Ok(())
}

struct CliOptions {
    port: u16,
    path: String,
    dns_name: Option<String>,
    token: Option<String>,
}

fn parse_args() -> Result<CliOptions> {
    let mut options = CliOptions {
        port: 8080,
        path: "/mcp".to_string(),
        dns_name: None,
        token: None,
    };
    let mut args = std::env::args().skip(1);

    while let Some(option) = args.next() {
        let value = || anyhow::anyhow!("Missing value for {option}");
        match option.as_str() {
            "--port" => {
                options.port = args
                    .next()
                    .ok_or_else(value)?
                    .parse::<u16>()
                    .context("Port must be a number between 0 and 65535")?;
            }
            "--path" => options.path = args.next().ok_or_else(value)?,
            "--dns-name" => options.dns_name = Some(args.next().ok_or_else(value)?),
            "--token" => options.token = Some(args.next().ok_or_else(value)?),
            "--help" | "-h" => {
                println!("dba-mcp-server {}", env!("CARGO_PKG_VERSION"));
                println!("Usage: dba-mcp-server [OPTIONS]\n");
                println!("Options:");
                println!("  -h, --help              Print help information");
                println!("  -V, --version           Print version information");
                println!("      --port <port>       HTTP port (default: 8080)");
                println!("      --path <path>       MCP endpoint path (default: /mcp)");
                println!("      --dns-name <name>   Add a DNS name to the allowed Host list");
                println!("      --token <token>     Set bearer token (generated if omitted)");
                println!("\nExample: dba-mcp-server --port 9000 --path /database-tools --dns-name dba.example.com");
                std::process::exit(0);
            }
            "--version" | "-V" => {
                println!("dba-mcp-server {}", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            _ => bail!("Unknown command-line option: {option}. Use --help for usage."),
        }
    }

    if let Some(name) = &options.dns_name {
        ensure!(!name.trim().is_empty(), "DNS name must not be empty");
    }
    if let Some(token) = &options.token {
        ensure!(
            !token.is_empty() && !token.chars().any(char::is_whitespace),
            "Token must be non-empty and contain no whitespace"
        );
    }

    Ok(options)
}

fn generate_token() -> Result<String> {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).context("Could not generate a secure bearer token")?;
    Ok(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
}
