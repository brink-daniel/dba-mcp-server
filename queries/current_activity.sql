SELECT
    r.session_id,
    s.login_name,
    s.host_name,
    s.program_name,
    DB_NAME(r.database_id) AS database_name,
    r.status AS request_status,
    r.command,
    r.start_time,
    r.total_elapsed_time AS elapsed_time_ms,
    r.cpu_time AS cpu_time_ms,
    r.logical_reads,
    r.reads AS physical_reads,
    r.writes,
    r.wait_type,
    r.wait_time AS wait_time_ms,
    r.wait_resource,
    r.blocking_session_id,
    SUBSTRING(
        sql_text.text,
        (r.statement_start_offset / 2) + 1,
        ((CASE r.statement_end_offset
            WHEN -1 THEN DATALENGTH(sql_text.text)
            ELSE r.statement_end_offset
        END - r.statement_start_offset) / 2) + 1
    ) AS statement_text,
    sql_text.text AS batch_text
FROM sys.dm_exec_requests AS r
JOIN sys.dm_exec_sessions AS s
    ON s.session_id = r.session_id
CROSS APPLY sys.dm_exec_sql_text(r.sql_handle) AS sql_text
WHERE s.is_user_process = 1
    AND r.session_id <> @@SPID
ORDER BY
    CASE WHEN r.blocking_session_id <> 0 THEN 0 ELSE 1 END,
    r.wait_time DESC,
    r.total_elapsed_time DESC;
