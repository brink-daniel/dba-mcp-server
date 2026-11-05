SELECT
    DB_NAME(files.database_id) AS database_name,
    files.name AS logical_file_name,
    files.type_desc AS file_type,
    files.physical_name,
    CONVERT(decimal(19, 2), files.size * 8.0 / 1024) AS allocated_size_mb,
    CASE
        WHEN files.max_size = -1 THEN NULL
        WHEN files.max_size = 0 THEN CONVERT(decimal(19, 2), 0)
        ELSE CONVERT(decimal(19, 2), files.max_size * 8.0 / 1024)
    END AS maximum_size_mb,
    CASE
        WHEN files.max_size = -1 THEN CAST(1 AS bit)
        ELSE CAST(0 AS bit)
    END AS unlimited_growth,
    files.is_percent_growth,
    CASE
        WHEN files.is_percent_growth = 1 THEN CONVERT(decimal(9, 2), files.growth)
        ELSE CONVERT(decimal(19, 2), files.growth * 8.0 / 1024)
    END AS autogrowth_value,
    CASE
        WHEN files.is_percent_growth = 1 THEN 'percent'
        ELSE 'mb'
    END AS autogrowth_unit
FROM sys.master_files AS files
ORDER BY DB_NAME(files.database_id), files.type_desc, files.file_id;
