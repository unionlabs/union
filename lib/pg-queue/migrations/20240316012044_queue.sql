-- CREATE TYPE status AS ENUM(
--     'ready',
--     'done',
--     'optimize',
--     'failed'
-- );

CREATE TABLE queue(
    id BIGSERIAL PRIMARY KEY,
    item JSONB NOT NULL,
    parents BIGINT[] DEFAULT '{}',
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE optimize(
    -- TODO: Figure out how to do this properly
    id BIGINT PRIMARY KEY DEFAULT nextval('queue_id_seq'::regclass),
    item JSONB NOT NULL,
    tag text NOT NULL,
    parents BIGINT[] DEFAULT '{}',
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE done(
    id BIGINT,
    item JSONB NOT NULL,
    parents BIGINT[] DEFAULT '{}',
    created_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (id, created_at)
);

CREATE TABLE failed(
    id BIGINT PRIMARY KEY,
    item JSONB NOT NULL,
    parents BIGINT[] DEFAULT '{}',
    message TEXT,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX index_queue_id ON queue(id);

SELECT
    create_hypertable('done', 'created_at');

SELECT
    add_retention_policy('done', INTERVAL '60 days');

ALTER TABLE done SET (timescaledb.compress, timescaledb.compress_orderby = 'created_at DESC', timescaledb.compress_segmentby = 'id');

SELECT
    add_compression_policy('done', compress_after => INTERVAL '14 days');
