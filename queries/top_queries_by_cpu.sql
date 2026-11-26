SELECT TOP (25)
    DB_NAME(sql_text.dbid) AS database_name,
    stats.execution_count,
    CONVERT(decimal(19, 2), stats.total_worker_time / 1000.0) AS total_cpu_ms,
    CONVERT(decimal(19, 2), stats.total_worker_time / NULLIF(stats.execution_count * 1000.0, 0)) AS average_cpu_ms,
    CONVERT(decimal(19, 2), stats.total_elapsed_time / NULLIF(stats.execution_count * 1000.0, 0)) AS average_elapsed_ms,
    stats.total_logical_reads,
    stats.total_logical_writes,
    stats.creation_time AS plan_creation_time,
    stats.last_execution_time,
    SUBSTRING(
        sql_text.text,
        (stats.statement_start_offset / 2) + 1,
        ((CASE stats.statement_end_offset
            WHEN -1 THEN DATALENGTH(sql_text.text)
            ELSE stats.statement_end_offset
        END - stats.statement_start_offset) / 2) + 1
    ) AS statement_text
FROM sys.dm_exec_query_stats AS stats
CROSS APPLY sys.dm_exec_sql_text(stats.sql_handle) AS sql_text
ORDER BY stats.total_worker_time DESC;
