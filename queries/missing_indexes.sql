SELECT TOP (50)
    DB_NAME(detail.database_id) AS database_name,
    OBJECT_SCHEMA_NAME(detail.object_id, detail.database_id) AS schema_name,
    OBJECT_NAME(detail.object_id, detail.database_id) AS table_name,
    CONVERT(decimal(19, 2), stats.avg_total_user_cost * stats.avg_user_impact *
        (stats.user_seeks + stats.user_scans)) AS estimated_impact,
    stats.user_seeks,
    stats.user_scans,
    stats.last_user_seek,
    stats.last_user_scan,
    detail.equality_columns,
    detail.inequality_columns,
    detail.included_columns
FROM sys.dm_db_missing_index_group_stats AS stats
JOIN sys.dm_db_missing_index_groups AS groups
    ON groups.index_group_handle = stats.group_handle
JOIN sys.dm_db_missing_index_details AS detail
    ON detail.index_handle = groups.index_handle
WHERE detail.database_id IS NOT NULL
ORDER BY estimated_impact DESC;
