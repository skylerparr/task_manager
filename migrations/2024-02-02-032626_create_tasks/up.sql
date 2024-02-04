CREATE TABLE tasks (
    id uuid PRIMARY KEY,
    job_type VARCHAR NOT NULL,
    status VARCHAR NOT NULL,
    result VARCHAR,
    lock_version INT NOT NULL DEFAULT 1,
    start_job_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX tasks_job_type ON tasks (job_type);
CREATE INDEX tasks_status ON tasks (status);
CREATE INDEX tasks_start_job_at ON tasks (start_job_at);
CREATE INDEX tasks_updated_at ON tasks (updated_at);
