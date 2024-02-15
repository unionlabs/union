CREATE TABLE queue (
    id BIGSERIAL PRIMARY KEY,
    status status NOT NULL DEFAULT 'ready',
    topic varchar(16) NOT NULL,
    item JSONB NOT NULL,
    -- Error message in case of permanent failure. If set, status MUST be 'failed'.
    message TEXT CHECK (((message IS NULL) AND (status != 'failed'::status)) OR ((message IS NOT NULL) AND (status = 'failed'::status))),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX index_queue_status ON queue (status);

CREATE INDEX idx_queue_id_created ON queue (created_at);

SELECT create_hypertable('queue', 'id')
