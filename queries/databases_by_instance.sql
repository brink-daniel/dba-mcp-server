SELECT
    database_id,
    name AS database_name,
    state_desc AS database_state,
    user_access_desc,
    recovery_model_desc,
    compatibility_level,
    collation_name,
    create_date,
    is_read_only,
    is_auto_close_on,
    is_auto_shrink_on,
    page_verify_option_desc,
    containment_desc,
    log_reuse_wait_desc
FROM sys.databases
ORDER BY name;
