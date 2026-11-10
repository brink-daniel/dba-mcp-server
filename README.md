# DBA MCP Server

Use the MCP client to check SQL Server health, activity, backups, SQL Agent jobs, error logs, and availability groups. The server runs on Windows and uses the Windows account that starts it to connect to SQL Server.

## Requirements

- Windows and the `dba-mcp-server.exe` executable.
- Network access from the server process to each target SQL Server instance.
- A Windows account with the SQL Server permissions required by the tools you use.

The server currently only supports Windows Integrated Authentication. 

## Run the server

Start the server from PowerShell with:

```powershell
.\dba-mcp-server.exe
```

The server listens on port `8080` at `/mcp` by default and prints its endpoint and bearer token at startup. Keep the token private.
The default endpoint is `http://localhost:8080/mcp`.

Available options:

| Option | Default | Purpose |
| --- | --- | --- |
| `--help`, `-h` | — | Prints command-line usage and exits. |
| `--version`, `-V` | — | Prints the package name and version, then exits. |
| `--port <port>` | `8080` | HTTP listening port. |
| `--path <path>` | `/mcp` | MCP endpoint path. |
| `--dns-name <name>` | none | Adds a DNS name to the MCP server's allowed Host list and prints its endpoint. |
| `--token <token>` | generated | Uses a predefined bearer token. It must be non-empty and contain no whitespace. |

For example, to use port `9000`, a custom path, and a DNS name:

```powershell
.\dba-mcp-server.exe --port 9000 --path /database-tools --dns-name dba.example.com
```

The corresponding endpoint would be `http://dba.example.com:9000/database-tools`. 

Clients must send `Authorization: Bearer <token>`. The server rejects requests without the correct token.

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

SQL Server access uses the Windows account running the server. Give that account a SQL Server login, access to the databases it needs, and only the read permissions for the tools you plan to use. The server does not need `sysadmin` or `db_owner`.

- **Database and file details:** `VIEW ANY DATABASE` and `VIEW ANY DEFINITION` at the server level. Seeing offline databases may require additional permissions.
- **Activity, disk, and availability groups:** `VIEW SERVER STATE` on SQL Server 2019 and earlier, or `VIEW SERVER PERFORMANCE STATE` on SQL Server 2022 and later. Availability group tools also need `VIEW ANY DEFINITION`.
- **Backups, restores, and SQL Agent:** grant `SELECT` on the relevant tables in `msdb` (`backupset`, `backupmediafamily`, `restorehistory`, `sysjobs`, `sysjobhistory`, `sysjobsteps`).
- **Error logs:** grant `EXECUTE` on `dbo.xp_readerrorlog` to a user in `master`. On SQL Server 2019 and earlier, also grant `VIEW SERVER STATE`; on SQL Server 2022 and later, grant `VIEW ANY ERROR LOG` or `VIEW SERVER PERFORMANCE STATE`.
- **Query Store execution reports:** grant `VIEW DATABASE STATE` (SQL Server 2019 and earlier) or `VIEW DATABASE PERFORMANCE STATE` (SQL Server 2022 and later) in each target database.

For example, these server-level grants enable common inventory and monitoring tools on SQL Server 2022 or later. Replace the login with the Windows account running the server:

```sql
USE [master];
GRANT VIEW ANY DATABASE TO [DOMAIN\service-account];
GRANT VIEW ANY DEFINITION TO [DOMAIN\service-account];
GRANT VIEW SERVER PERFORMANCE STATE TO [DOMAIN\service-account];
```

Use `VIEW SERVER STATE` in place of `VIEW SERVER PERFORMANCE STATE` on SQL Server 2019 and earlier. Database-level access requires a user mapped to the login in each database. Actual access can also depend on SQL Server version, explicit denies, and enabled features.

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
| `get_query_execution_aborted` | `instance`, `database`, optional `start_datetime`, optional `end_datetime` | Reads Query Store client-aborted execution aggregates from intervals overlapping the range. Defaults to the previous hour through now. |
| `get_query_execution_exception` | `instance`, `database`, optional `start_datetime`, optional `end_datetime` | Reads Query Store execution exception aggregates from intervals overlapping the range. Defaults to the previous hour through now. |
| `get_availability_group_status` | `instance` | Reports availability group replica roles, operational/connection/health state, last connection error, and send/redo queue sizes. |


