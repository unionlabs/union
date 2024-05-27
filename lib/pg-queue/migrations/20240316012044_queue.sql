CREATE TABLE queue (
    id BIGSERIAL,
    status status NOT NULL DEFAULT 'ready',
    item JSONB NOT NULL,
    -- Can't have foreign key relations to hypertables, so recreate the constraints as best as possible
    parent BIGINT DEFAULT NULL CHECK (parent IS NULL OR parent > 0),
    -- Error message in case of permanent failure. If set, status MUST be 'failed'.
    message TEXT CHECK (((message IS NULL) AND (status != 'failed'::status)) OR ((message IS NOT NULL) AND (status = 'failed'::status))),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (id, created_at)
);

CREATE INDEX index_queue_id ON queue (id);

CREATE INDEX index_queue_status_id ON queue (status, id);

SELECT create_hypertable('queue', 'created_at');

SELECT add_retention_policy('queue', INTERVAL '60 days');

ALTER TABLE queue SET (timescaledb.compress, timescaledb.compress_orderby = 'created_at DESC', timescaledb.compress_segmentby = 'id');

SELECT add_compression_policy('queue', compress_after => INTERVAL '14 days');
