SELECT DISTINCT
    volume.volume_mount_point,
    volume.logical_volume_name,
    CONVERT(decimal(19, 2), volume.total_bytes / 1073741824.0) AS total_size_gb,
    CONVERT(decimal(19, 2), volume.available_bytes / 1073741824.0) AS available_space_gb,
    CONVERT(decimal(5, 2), 100.0 * volume.available_bytes / NULLIF(volume.total_bytes, 0)) AS available_space_percent
FROM sys.master_files AS files
CROSS APPLY sys.dm_os_volume_stats(files.database_id, files.file_id) AS volume
ORDER BY volume.volume_mount_point;
