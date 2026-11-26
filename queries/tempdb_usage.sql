WITH tempdb_files AS (
    SELECT
        SUM(unallocated_extent_page_count) AS free_pages,
        SUM(version_store_reserved_page_count) AS version_store_pages,
        SUM(user_object_reserved_page_count) AS user_object_pages,
        SUM(internal_object_reserved_page_count) AS internal_object_pages
    FROM sys.dm_db_file_space_usage
), session_usage AS (
    SELECT
        session_id,
        SUM(user_objects_alloc_page_count + internal_objects_alloc_page_count) AS allocated_pages,
        SUM(user_objects_dealloc_page_count + internal_objects_dealloc_page_count) AS deallocated_pages
    FROM sys.dm_db_session_space_usage
    GROUP BY session_id
)
SELECT TOP (50)
    sessions.session_id,
    sessions.login_name,
    sessions.host_name,
    sessions.program_name,
    CONVERT(decimal(19, 2), files.free_pages * 8.0 / 1024) AS tempdb_free_mb,
    CONVERT(decimal(19, 2), files.version_store_pages * 8.0 / 1024) AS version_store_mb,
    CONVERT(decimal(19, 2), files.user_object_pages * 8.0 / 1024) AS user_object_mb,
    CONVERT(decimal(19, 2), files.internal_object_pages * 8.0 / 1024) AS internal_object_mb,
    CONVERT(decimal(19, 2), usage.allocated_pages * 8.0 / 1024) AS session_allocated_mb,
    CONVERT(decimal(19, 2), usage.deallocated_pages * 8.0 / 1024) AS session_deallocated_mb
FROM sys.dm_exec_sessions AS sessions
JOIN session_usage AS usage
    ON usage.session_id = sessions.session_id
CROSS JOIN tempdb_files AS files
WHERE sessions.is_user_process = 1
    AND (usage.allocated_pages > 0 OR usage.deallocated_pages > 0)
ORDER BY usage.allocated_pages DESC;
