SELECT
    b.database_name,
    CASE b.type
        WHEN 'D' THEN 'Full'
        WHEN 'I' THEN 'Differential'
        WHEN 'L' THEN 'Transaction Log'
        WHEN 'F' THEN 'File/Filegroup'
        WHEN 'G' THEN 'Differential File'
        WHEN 'P' THEN 'Partial'
        WHEN 'Q' THEN 'Differential Partial'
        ELSE b.type
    END AS backup_type,
    b.backup_start_date,
    b.backup_finish_date,
    b.backup_size,
    b.compressed_backup_size,
    b.server_name,
    m.physical_device_name
FROM dbo.backupset AS b
LEFT JOIN dbo.backupmediafamily AS m
    ON m.media_set_id = b.media_set_id
WHERE b.database_name = @P1
ORDER BY b.backup_finish_date DESC;
