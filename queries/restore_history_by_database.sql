SELECT
    rh.destination_database_name AS database_name,
    rh.restore_date,
    rh.restore_type,
    CASE rh.restore_type
        WHEN 'D' THEN 'Database'
        WHEN 'F' THEN 'File/Filegroup'
        WHEN 'G' THEN 'Filegroup'
        WHEN 'I' THEN 'Differential'
        WHEN 'L' THEN 'Transaction Log'
        WHEN 'V' THEN 'Verify-only'
        WHEN 'R' THEN 'Revert'
        ELSE rh.restore_type
    END AS restore_type_description,
    rh.replace,
    rh.recovery,
    rh.restart,
    rh.stop_at,
    rh.user_name,
    b.backup_set_id,
    b.backup_start_date AS source_backup_start_date,
    b.backup_finish_date AS source_backup_finish_date
FROM dbo.restorehistory AS rh
LEFT JOIN dbo.backupset AS b
    ON b.backup_set_id = rh.backup_set_id
WHERE rh.destination_database_name = @P1
ORDER BY rh.restore_date DESC;
