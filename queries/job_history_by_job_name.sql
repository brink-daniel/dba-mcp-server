SELECT
    j.name AS job_name,
    h.step_id,
    COALESCE(s.step_name, '(Job outcome)') AS step_name,
    h.run_date,
    h.run_time,
    h.run_duration,
    h.run_status,
    h.sql_severity,
    h.message
FROM dbo.sysjobhistory AS h
INNER JOIN dbo.sysjobs AS j
    ON j.job_id = h.job_id
LEFT JOIN dbo.sysjobsteps AS s
    ON s.job_id = h.job_id
    AND s.step_id = h.step_id
WHERE j.name = @P1
ORDER BY h.instance_id DESC;
