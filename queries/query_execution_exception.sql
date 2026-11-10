SELECT
    q.query_id,
    p.plan_id,
    qt.query_sql_text,
    rsi.start_time AS query_store_interval_start,
    rsi.end_time AS query_store_interval_end,
    rs.execution_type_desc,
    SUM(rs.count_executions) AS exception_execution_count,
    CONVERT(decimal(19, 2),
        SUM(rs.avg_duration * rs.count_executions)
        / NULLIF(SUM(rs.count_executions), 0)) AS weighted_avg_duration_us,
    MAX(rs.max_duration) AS max_duration_us
FROM sys.query_store_runtime_stats AS rs
JOIN sys.query_store_plan AS p
    ON p.plan_id = rs.plan_id
JOIN sys.query_store_query AS q
    ON q.query_id = p.query_id
JOIN sys.query_store_query_text AS qt
    ON qt.query_text_id = q.query_text_id
JOIN sys.query_store_runtime_stats_interval AS rsi
    ON rsi.runtime_stats_interval_id = rs.runtime_stats_interval_id
WHERE rs.execution_type = 4 -- Exception
    AND rsi.start_time < @P2
    AND rsi.end_time > @P1
GROUP BY
    q.query_id,
    p.plan_id,
    qt.query_sql_text,
    rsi.start_time,
    rsi.end_time,
    rs.execution_type_desc
ORDER BY rsi.start_time DESC, exception_execution_count DESC;
