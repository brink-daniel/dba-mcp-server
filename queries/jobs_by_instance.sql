SELECT
    job_id,
    name,
    enabled,
    description,
    date_created,
    date_modified
FROM dbo.sysjobs
ORDER BY name;
