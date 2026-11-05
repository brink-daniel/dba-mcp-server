WITH job_runs AS (
    SELECT
        j.name AS job_name,
        h.run_status,
        h.message,
        msdb.dbo.agent_datetime(h.run_date, h.run_time) AS run_start,
        DATEADD(
            SECOND,
            (h.run_duration / 10000) * 3600
                + ((h.run_duration % 10000) / 100) * 60
                + (h.run_duration % 100),
            msdb.dbo.agent_datetime(h.run_date, h.run_time)
        ) AS run_end
    FROM dbo.sysjobhistory AS h
    INNER JOIN dbo.sysjobs AS j
        ON j.job_id = h.job_id
    WHERE h.step_id = 0
)
SELECT
    job_name,
    run_start,
    run_end,
    run_status,
    message
FROM job_runs
WHERE run_start <= @P2
    AND run_end >= @P1
ORDER BY run_start DESC;
