SELECT
    ag.name AS availability_group_name,
    ar.replica_server_name,
    ars.is_local,
    ars.role_desc,
    ars.operational_state_desc,
    ars.connected_state_desc,
    ars.synchronization_health_desc,
    ars.last_connect_error_description,
    dbs.log_send_queue_size_kb,
    dbs.redo_queue_size_kb
FROM sys.availability_groups AS ag
JOIN sys.availability_replicas AS ar
    ON ar.group_id = ag.group_id
LEFT JOIN sys.dm_hadr_availability_replica_states AS ars
    ON ars.group_id = ag.group_id
    AND ars.replica_id = ar.replica_id
OUTER APPLY (
    SELECT
        SUM(CONVERT(bigint, drs.log_send_queue_size)) AS log_send_queue_size_kb,
        SUM(CONVERT(bigint, drs.redo_queue_size)) AS redo_queue_size_kb
    FROM sys.dm_hadr_database_replica_states AS drs
    WHERE drs.group_id = ag.group_id
        AND drs.replica_id = ar.replica_id
) AS dbs
ORDER BY ag.name, ar.replica_server_name;
