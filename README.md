# DBA MCP Server

A read-only Model Context Protocol (MCP) server for SQL Server administration and diagnostics. It serves MCP over Streamable HTTP and connects to SQL Server with the Windows identity running the server.

## Requirements

- Windows and a Rust toolchain that supports the 2024 edition.
- Network access from the server process to each target SQL Server.
- A Windows account with the SQL Server permissions required by the tools you use.

The server currently uses Windows Integrated Authentication. SQL Server connections on non-Windows platforms return an error.

## Run the server

From the repository directory:

```powershell
cargo run
```

Defaults are port `8080` and MCP path `/mcp`. The server binds to `0.0.0.0`, so it listens on all network interfaces. It generates a 32-byte cryptographically random bearer token at startup unless `--token` is supplied. The token is printed to standard output with the endpoint URLs; treat it as a secret.

Available options:

| Option | Default | Purpose |
| --- | --- | --- |
| `--help`, `-h` | — | Prints command-line usage and exits. |
| `--version`, `-V` | — | Prints the package name and version, then exits. |
| `--port <port>` | `8080` | HTTP listening port. |
| `--path <path>` | `/mcp` | MCP endpoint path. |
| `--dns-name <name>` | none | Adds a DNS name to the MCP server's allowed Host list and prints its endpoint. |
| `--token <token>` | generated | Uses a predefined bearer token. It must be non-empty and contain no whitespace. |

For example:

```powershell
cargo run -- --port 9000 --path /database-tools --dns-name dba.example.com
```

The corresponding endpoint is `http://localhost:9000/database-tools` (also printed for the computer name and configured DNS name when available). The default endpoint is `http://localhost:8080/mcp`.

Every request must include `Authorization: Bearer <token>`. Requests with a missing or incorrect token receive HTTP `401 Unauthorized`.

### Connect a client

The repository contains two client configurations for the same default endpoint:

- **Codex:** `.codex/config.toml` registers `dba_mcp_server` and reads the token from the `DBA_MCP_TOKEN` environment variable. Set that variable in Windows, then fully restart Codex so it inherits the value and reloads the project MCP configuration. The current Codex chat does not prompt for this token.
- **Clients supporting this `mcp.json` input convention:** `mcp.json` registers `dba-mcp-server` and prompts for `dba-mcp-token`. Enter the token printed by the running server when that client requests it.

If you change the server port or path, update the endpoint in the client configuration you use. Do not commit a real token to either configuration file.

### Network security

The server uses plain HTTP and allows cross-origin requests. Bearer authentication does not encrypt traffic. Keep the listener on a trusted network or place it behind a trusted TLS-terminating proxy and restrict network access before exposing it outside a trusted local environment.

## Connect to SQL Server

Every tool takes an `instance` argument with a host name and optional TCP port. The default port is `1433`:

- `localhost`
- `sqlserver.example.com`
- `sqlserver.example.com:1433`

The SQL Server certificate is trusted by the client configuration. See [SQL Server permissions](#sql-server-permissions) for the per-tool grants and version differences.

## SQL Server permissions

The server connects as the Windows identity running the MCP process. That identity must have a SQL Server login and access to the databases queried below. These grants cover the objects read by the repository's scripts; do not grant `sysadmin` or `db_owner` just to use this server. Replace `[DOMAIN\service-account]` with the Windows login. Apply only the version-specific server-state grant that applies.

| Tool(s) | Required access |
| --- | --- |
| `get_databases` | `VIEW ANY DATABASE` to see all databases. SQL Server grants this to `public` by default. If that default has been revoked or denied, grant it to the login. Offline database rows have stricter visibility: `ALTER ANY DATABASE` or `CREATE DATABASE` in `master` is required to see them. |
| `get_current_activity` | SQL Server 2019 and earlier: server permission `VIEW SERVER STATE`. SQL Server 2022 and later: `VIEW SERVER PERFORMANCE STATE`. The query reads `sys.dm_exec_requests`, `sys.dm_exec_sessions`, and `sys.dm_exec_sql_text`; the principal needs `SELECT` on those objects if not otherwise granted. |
| `get_disk_space` | `VIEW ANY DEFINITION` to see all rows in `sys.master_files`, plus `VIEW SERVER STATE` (SQL Server 2019 and earlier) or `VIEW SERVER PERFORMANCE STATE` (SQL Server 2022 and later) for `sys.dm_os_volume_stats`. The principal also needs `SELECT` on the DMV/DMF if not otherwise granted. |
| `get_database_file_usage` | `VIEW ANY DEFINITION` to see all rows in `sys.master_files`. The catalog view also lists `ALTER ANY DATABASE` or `CREATE DATABASE` as alternatives; those are broader write-capable permissions and are not recommended for this read-only use. |
| `get_backup_history_by_database` | In `msdb`, `SELECT` on `dbo.backupset` and `dbo.backupmediafamily`. |
| `get_restore_history_by_database` | In `msdb`, `SELECT` on `dbo.restorehistory` and `dbo.backupset`. |
| `get_sqlagent_jobs` | In `msdb`, `SELECT` on `dbo.sysjobs`. |
| `get_sqlagent_job_history_by_job_name` | In `msdb`, `SELECT` on `dbo.sysjobs`, `dbo.sysjobhistory`, and `dbo.sysjobsteps`. |
| `get_sqlagent_jobs_running_between_start_end_time` | In `msdb`, `SELECT` on `dbo.sysjobs` and `dbo.sysjobhistory`. |
| `get_server_errorlog`, `get_agent_errorlog` | The scripts call `master.dbo.xp_readerrorlog` directly, so grant `EXECUTE` on that object to a database user in `master`. Also grant `VIEW SERVER STATE` on SQL Server 2019 and earlier. On SQL Server 2022 and later, grant `VIEW ANY ERROR LOG` (the narrower choice for log-only access) or `VIEW SERVER PERFORMANCE STATE`. |
| `get_timeouts` | In each database queried, grant `VIEW DATABASE STATE` on SQL Server 2019 and earlier, or `VIEW DATABASE PERFORMANCE STATE` on SQL Server 2022 and later. The query reads `sys.query_store_runtime_stats`, `sys.query_store_plan`, `sys.query_store_query`, `sys.query_store_query_text`, and `sys.query_store_runtime_stats_interval`. Query Store must be enabled. |
| `get_availability_group_status` | SQL Server 2019 and earlier: `VIEW ANY DEFINITION` and `VIEW SERVER STATE`. SQL Server 2022 and later: `VIEW ANY DEFINITION` and `VIEW SERVER PERFORMANCE STATE`. The query reads `sys.availability_groups`, `sys.availability_replicas`, `sys.dm_hadr_availability_replica_states`, and `sys.dm_hadr_database_replica_states`; grant `SELECT` on the DMVs if not otherwise granted. Availability Groups must be enabled and configured for results to be returned. |

For example, grant the server-scoped read permissions needed for database inventory, file metadata, and current activity on SQL Server 2022 or later:

```sql
USE [master];
GRANT VIEW ANY DATABASE TO [DOMAIN\service-account];
GRANT VIEW ANY DEFINITION TO [DOMAIN\service-account];
GRANT VIEW SERVER PERFORMANCE STATE TO [DOMAIN\service-account];
```

On SQL Server 2019 or earlier, replace the last grant with:

```sql
GRANT VIEW SERVER STATE TO [DOMAIN\service-account];
```

For SQL Server 2022 or later, `VIEW ANY ERROR LOG` is narrower than `VIEW SERVER PERFORMANCE STATE` when granting error-log access only. The login must also have a user in `master` for the object-level grant. Grant the direct extended-procedure permission in `master` for both log tools:

```sql
USE [master];
-- Create this database user only if it does not already exist.
CREATE USER [DOMAIN\service-account] FOR LOGIN [DOMAIN\service-account];
GRANT VIEW ANY ERROR LOG TO [DOMAIN\service-account];
GRANT EXECUTE ON OBJECT::dbo.xp_readerrorlog TO [DOMAIN\service-account];
```

Grant `SELECT` only on the `msdb` objects required for the tools being enabled. For example, to enable backup, restore, and SQL Agent history/jobs:

```sql
USE [msdb];
-- Create this database user only if it does not already exist.
CREATE USER [DOMAIN\service-account] FOR LOGIN [DOMAIN\service-account];
GRANT SELECT ON OBJECT::dbo.backupset TO [DOMAIN\service-account];
GRANT SELECT ON OBJECT::dbo.backupmediafamily TO [DOMAIN\service-account];
GRANT SELECT ON OBJECT::dbo.restorehistory TO [DOMAIN\service-account];
GRANT SELECT ON OBJECT::dbo.sysjobs TO [DOMAIN\service-account];
GRANT SELECT ON OBJECT::dbo.sysjobhistory TO [DOMAIN\service-account];
GRANT SELECT ON OBJECT::dbo.sysjobsteps TO [DOMAIN\service-account];
```

Map the login to a user in `msdb` before granting these object permissions. For Query Store, map the login to a user in each target database and grant the version-appropriate database permission there. The login must be able to connect to each database it queries; the standard `public` `CONNECT` grants usually provide this unless changed. `SQLAgentReaderRole` is a built-in alternative for SQL Agent visibility, but it also grants job-management capabilities for jobs owned by that user; object-level `SELECT` grants are narrower for this server's read-only queries.

For Query Store on SQL Server 2022 or later, the database-specific grant looks like this (use `VIEW DATABASE STATE` instead on SQL Server 2019 and earlier):

```sql
USE [target_database];
-- Create this database user only if it does not already exist.
CREATE USER [DOMAIN\service-account] FOR LOGIN [DOMAIN\service-account];
GRANT VIEW DATABASE PERFORMANCE STATE TO [DOMAIN\service-account];
```

These are the minimum permissions implied by the statements in this repository, not a guarantee that every environment uses default system-object permissions. Explicit `DENY` entries, revoked `public` permissions, database state, SQL Server version, or disabled features can still prevent results. For database-scoped objects, the principal must be mapped to a database user and have `CONNECT`; a DMV/DMF can require both its state permission and `SELECT` on the object. Microsoft documents the relevant requirements for [`sys.databases`](https://learn.microsoft.com/en-us/sql/relational-databases/system-catalog-views/sys-databases-transact-sql), [`sys.master_files`](https://learn.microsoft.com/en-us/sql/relational-databases/system-catalog-views/sys-master-files-transact-sql), [server/database DMVs](https://learn.microsoft.com/en-us/sql/relational-databases/system-dynamic-management-views/system-dynamic-management-views), [Query Store views](https://learn.microsoft.com/en-us/sql/relational-databases/system-catalog-views/sys-query-store-runtime-stats-transact-sql), [error-log reading](https://learn.microsoft.com/en-us/sql/relational-databases/system-stored-procedures/sp-readerrorlog-transact-sql), [SQL Server Agent roles](https://learn.microsoft.com/en-us/ssms/agent/sql-server-agent-fixed-database-roles), and [Always On monitoring](https://learn.microsoft.com/en-us/sql/database-engine/availability-groups/windows/monitor-availability-groups-transact-sql).

The error-log scripts call the undocumented extended procedure `xp_readerrorlog`, not Microsoft's documented `sp_readerrorlog` wrapper. The explicit `EXECUTE` grant above matches the object the scripts invoke; the version-specific log permission follows Microsoft's published error-log permission guidance. If a SQL Server build still rejects the direct extended-procedure call, migrate the query to the documented wrapper and use its documented permissions.

## Tools

All tools are read-only.

| Tool | Arguments | Description |
| --- | --- | --- |
| `get_databases` | `instance` | Lists database names, states, access, recovery model, compatibility, collation, and selected database settings. |
| `get_current_activity` | `instance` | Shows active user requests, session details, waits, blockers, resource counts, SQL statement, and batch text. |
| `get_disk_space` | `instance` | Reports total and available space for volumes containing SQL Server database files. |
| `get_database_file_usage` | `instance` | Reports allocated file size, maximum size, autogrowth, and file path. Allocated size is not used-page space. |
| `get_backup_history_by_database` | `instance`, `database` | Reads backup history from `msdb`, including backup type, start/finish, sizes, server, and media path. |
| `get_restore_history_by_database` | `instance`, `database` | Reads restore history from `msdb`, including destination, restore type, date, options, user, and source backup times. |
| `get_server_errorlog` | `instance`, optional `log_number` | Reads a SQL Server error log. `log_number` defaults to `0` (current); `1` is the previous log. |
| `get_agent_errorlog` | `instance`, optional `log_number` | Reads a SQL Server Agent error log. `log_number` defaults to `0` (current); `1` is the previous log. |
| `get_sqlagent_jobs` | `instance` | Lists SQL Server Agent jobs and basic metadata. |
| `get_sqlagent_job_history_by_job_name` | `instance`, `job_name` | Reads execution history for a named job, including step/outcome, status, duration, and message. |
| `get_sqlagent_jobs_running_between_start_end_time` | `instance`, `start_datetime`, `end_datetime` | Lists job runs that overlap a time range. Supply ISO 8601 date-time values in `YYYY-MM-DDTHH:MM:SS` form; fractional seconds are accepted. |
| `get_timeouts` | `instance`, `database`, optional `start_datetime`, optional `end_datetime` | Reads client-aborted Query Store execution aggregates from intervals overlapping the range. Defaults to the previous hour through now. |
| `get_availability_group_status` | `instance` | Reports availability group replica roles, operational/connection/health state, last connection error, and send/redo queue sizes. |

### Query Store limitation

Query Store records client-aborted executions but does not say whether an abort was specifically caused by a timeout. Runtime statistics are aggregated by interval, so `get_timeouts` returns possible timeout candidates and interval-level aggregates, not individual timeout events. Query Store must be enabled for the target database.

## Project layout

- `src/main.rs` — command-line options, bearer authentication, and Streamable HTTP server startup.
- `src/db.rs` — SQL Server connection, query execution, and conversion of result rows to JSON.
- `src/tools/` — MCP tool registrations and argument handling.
- `queries/` — SQL statements embedded by the corresponding tool modules.
- `.codex/config.toml` — Codex project MCP registration and token environment-variable name.
- `mcp.json` — sample MCP client configuration with a prompted token input.
- `.agents/skills/dba-mcp-server/SKILL.md` — instructions for using the project's DBA MCP tools.

## Build

```powershell
cargo build
cargo build --release
```
