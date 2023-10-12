SET check_function_bodies = false;
CREATE EXTENSION IF NOT EXISTS timescaledb WITH SCHEMA public;
COMMENT ON EXTENSION timescaledb IS 'Enables scalable inserts and complex queries for time-series data (Community Edition)';
CREATE EXTENSION IF NOT EXISTS timescaledb_toolkit WITH SCHEMA public;
COMMENT ON EXTENSION timescaledb_toolkit IS 'Library of analytical hyperfunctions, time-series pipelining, and other SQL utilities';
CREATE SCHEMA v0;
CREATE EXTENSION IF NOT EXISTS pg_stat_statements WITH SCHEMA public;
COMMENT ON EXTENSION pg_stat_statements IS 'track planning and execution statistics of all SQL statements executed';
CREATE FUNCTION v0.set_current_timestamp_updated_at() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
DECLARE
  _new record;
BEGIN
  _new := NEW;
  _new."updated_at" = NOW();
  RETURN _new;
END;
$$;
CREATE TABLE v0.events (
    block_id bigint NOT NULL,
    index integer NOT NULL,
    data jsonb NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    "time" timestamp with time zone NOT NULL
);
CREATE VIEW _timescaledb_internal._direct_view_5 AS
 SELECT (((events.data -> 'attributes'::text) -> 0) ->> 'value'::text) AS receiver,
    sum(("substring"((((events.data -> 'attributes'::text) -> 1) ->> 'value'::text), '^\d+'::text))::numeric) AS sum,
    public.time_bucket('00:30:00'::interval, events."time") AS period
   FROM v0.events
  WHERE ((events.data ->> 'type'::text) = 'coin_received'::text)
  GROUP BY (public.time_bucket('00:30:00'::interval, events."time")), (((events.data -> 'attributes'::text) -> 0) ->> 'value'::text);
CREATE TABLE _timescaledb_internal._hyper_2_1_chunk (
    CONSTRAINT constraint_1 CHECK ((("time" >= '2023-10-05 00:00:00+00'::timestamp with time zone) AND ("time" < '2023-10-12 00:00:00+00'::timestamp with time zone)))
)
INHERITS (v0.events);
CREATE TABLE _timescaledb_internal._hyper_2_2_chunk (
    CONSTRAINT constraint_2 CHECK ((("time" >= '2023-10-12 00:00:00+00'::timestamp with time zone) AND ("time" < '2023-10-19 00:00:00+00'::timestamp with time zone)))
)
INHERITS (v0.events);
CREATE TABLE _timescaledb_internal._materialized_hypertable_5 (
    receiver text,
    sum numeric,
    period timestamp with time zone NOT NULL
);
CREATE TABLE _timescaledb_internal._hyper_5_5_chunk (
    CONSTRAINT constraint_5 CHECK (((period >= '2023-08-31 00:00:00+00'::timestamp with time zone) AND (period < '2023-11-09 00:00:00+00'::timestamp with time zone)))
)
INHERITS (_timescaledb_internal._materialized_hypertable_5);
CREATE VIEW _timescaledb_internal._partial_view_5 AS
 SELECT (((events.data -> 'attributes'::text) -> 0) ->> 'value'::text) AS receiver,
    sum(("substring"((((events.data -> 'attributes'::text) -> 1) ->> 'value'::text), '^\d+'::text))::numeric) AS sum,
    public.time_bucket('00:30:00'::interval, events."time") AS period
   FROM v0.events
  WHERE ((events.data ->> 'type'::text) = 'coin_received'::text)
  GROUP BY (public.time_bucket('00:30:00'::interval, events."time")), (((events.data -> 'attributes'::text) -> 0) ->> 'value'::text);
CREATE TABLE v0.blocks (
    chain_id integer NOT NULL,
    id bigint NOT NULL,
    hash text NOT NULL,
    data jsonb NOT NULL,
    height bigint NOT NULL,
    "time" timestamp with time zone NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    is_finalized boolean DEFAULT false NOT NULL
);
CREATE SEQUENCE v0.blocks_block_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;
ALTER SEQUENCE v0.blocks_block_id_seq OWNED BY v0.blocks.id;
CREATE TABLE v0.chains (
    id integer NOT NULL,
    name text NOT NULL,
    chain_id text NOT NULL,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now()
);
CREATE SEQUENCE v0.chains_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;
ALTER SEQUENCE v0.chains_id_seq OWNED BY v0.chains.id;
CREATE VIEW v0.coin_receiveds AS
 SELECT events.block_id,
    events.index,
    events."time",
    (((events.data -> 'attributes'::text) -> 0) ->> 'value'::text) AS receiver,
    ("substring"((((events.data -> 'attributes'::text) -> 1) ->> 'value'::text), '^\d+'::text))::numeric AS amount,
    "substring"((((events.data -> 'attributes'::text) -> 1) ->> 'value'::text), '[a-zA-Z].*'::text) AS denom
   FROM v0.events
  WHERE ((events.data ->> 'type'::text) = 'coin_received'::text);
CREATE VIEW v0.coin_receiveds_30m AS
 SELECT _materialized_hypertable_5.receiver,
    _materialized_hypertable_5.sum,
    _materialized_hypertable_5.period
   FROM _timescaledb_internal._materialized_hypertable_5
  WHERE (_materialized_hypertable_5.period < COALESCE(_timescaledb_functions.to_timestamp(_timescaledb_functions.cagg_watermark(5)), '-infinity'::timestamp with time zone))
UNION ALL
 SELECT (((events.data -> 'attributes'::text) -> 0) ->> 'value'::text) AS receiver,
    sum(("substring"((((events.data -> 'attributes'::text) -> 1) ->> 'value'::text), '^\d+'::text))::numeric) AS sum,
    public.time_bucket('00:30:00'::interval, events."time") AS period
   FROM v0.events
  WHERE (((events.data ->> 'type'::text) = 'coin_received'::text) AND (events."time" >= COALESCE(_timescaledb_functions.to_timestamp(_timescaledb_functions.cagg_watermark(5)), '-infinity'::timestamp with time zone)))
  GROUP BY (public.time_bucket('00:30:00'::interval, events."time")), (((events.data -> 'attributes'::text) -> 0) ->> 'value'::text);
CREATE VIEW v0.coin_receiveds_totals AS
 SELECT sum(coin_receiveds_30m.sum) AS sum,
    coin_receiveds_30m.receiver
   FROM v0.coin_receiveds_30m
  GROUP BY coin_receiveds_30m.receiver;
ALTER TABLE ONLY _timescaledb_internal._hyper_2_1_chunk ALTER COLUMN created_at SET DEFAULT now();
ALTER TABLE ONLY _timescaledb_internal._hyper_2_1_chunk ALTER COLUMN updated_at SET DEFAULT now();
ALTER TABLE ONLY _timescaledb_internal._hyper_2_2_chunk ALTER COLUMN created_at SET DEFAULT now();
ALTER TABLE ONLY _timescaledb_internal._hyper_2_2_chunk ALTER COLUMN updated_at SET DEFAULT now();
ALTER TABLE ONLY v0.blocks ALTER COLUMN id SET DEFAULT nextval('v0.blocks_block_id_seq'::regclass);
ALTER TABLE ONLY v0.chains ALTER COLUMN id SET DEFAULT nextval('v0.chains_id_seq'::regclass);
ALTER TABLE ONLY _timescaledb_internal._hyper_2_1_chunk
    ADD CONSTRAINT "1_2_events_pkey" PRIMARY KEY (block_id, index, "time");
ALTER TABLE ONLY _timescaledb_internal._hyper_2_2_chunk
    ADD CONSTRAINT "2_4_events_pkey" PRIMARY KEY (block_id, index, "time");
ALTER TABLE ONLY v0.blocks
    ADD CONSTRAINT blocks_hash_chain_id_key UNIQUE (hash, chain_id);
ALTER TABLE ONLY v0.blocks
    ADD CONSTRAINT blocks_pkey PRIMARY KEY (id);
ALTER TABLE ONLY v0.chains
    ADD CONSTRAINT chains_pkey PRIMARY KEY (id);
ALTER TABLE ONLY v0.events
    ADD CONSTRAINT events_pkey PRIMARY KEY (block_id, index, "time");
CREATE INDEX _hyper_2_1_chunk_events_time_idx ON _timescaledb_internal._hyper_2_1_chunk USING btree ("time" DESC);
CREATE INDEX _hyper_2_2_chunk_events_time_idx ON _timescaledb_internal._hyper_2_2_chunk USING btree ("time" DESC);
CREATE INDEX _hyper_5_5_chunk__materialized_hypertable_5_period_idx ON _timescaledb_internal._hyper_5_5_chunk USING btree (period DESC);
CREATE INDEX _hyper_5_5_chunk__materialized_hypertable_5_receiver_period_idx ON _timescaledb_internal._hyper_5_5_chunk USING btree (receiver, period DESC);
CREATE INDEX _materialized_hypertable_5_period_idx ON _timescaledb_internal._materialized_hypertable_5 USING btree (period DESC);
CREATE INDEX _materialized_hypertable_5_receiver_period_idx ON _timescaledb_internal._materialized_hypertable_5 USING btree (receiver, period DESC);
CREATE INDEX events_time_idx ON v0.events USING btree ("time" DESC);
CREATE TRIGGER set_v0_events_updated_at BEFORE UPDATE ON _timescaledb_internal._hyper_2_1_chunk FOR EACH ROW EXECUTE FUNCTION v0.set_current_timestamp_updated_at();
CREATE TRIGGER set_v0_events_updated_at BEFORE UPDATE ON _timescaledb_internal._hyper_2_2_chunk FOR EACH ROW EXECUTE FUNCTION v0.set_current_timestamp_updated_at();
CREATE TRIGGER ts_cagg_invalidation_trigger AFTER INSERT OR DELETE OR UPDATE ON _timescaledb_internal._hyper_2_1_chunk FOR EACH ROW EXECUTE FUNCTION _timescaledb_functions.continuous_agg_invalidation_trigger('2');
CREATE TRIGGER ts_cagg_invalidation_trigger AFTER INSERT OR DELETE OR UPDATE ON _timescaledb_internal._hyper_2_2_chunk FOR EACH ROW EXECUTE FUNCTION _timescaledb_functions.continuous_agg_invalidation_trigger('2');
CREATE TRIGGER ts_insert_blocker BEFORE INSERT ON _timescaledb_internal._materialized_hypertable_5 FOR EACH ROW EXECUTE FUNCTION _timescaledb_functions.insert_blocker();
CREATE TRIGGER set_v0_blocks_updated_at BEFORE UPDATE ON v0.blocks FOR EACH ROW EXECUTE FUNCTION v0.set_current_timestamp_updated_at();
COMMENT ON TRIGGER set_v0_blocks_updated_at ON v0.blocks IS 'trigger to set value of column "updated_at" to current timestamp on row update';
CREATE TRIGGER set_v0_chains_updated_at BEFORE UPDATE ON v0.chains FOR EACH ROW EXECUTE FUNCTION v0.set_current_timestamp_updated_at();
COMMENT ON TRIGGER set_v0_chains_updated_at ON v0.chains IS 'trigger to set value of column "updated_at" to current timestamp on row update';
CREATE TRIGGER set_v0_events_updated_at BEFORE UPDATE ON v0.events FOR EACH ROW EXECUTE FUNCTION v0.set_current_timestamp_updated_at();
COMMENT ON TRIGGER set_v0_events_updated_at ON v0.events IS 'trigger to set value of column "updated_at" to current timestamp on row update';
CREATE TRIGGER ts_cagg_invalidation_trigger AFTER INSERT OR DELETE OR UPDATE ON v0.events FOR EACH ROW EXECUTE FUNCTION _timescaledb_functions.continuous_agg_invalidation_trigger('2');
CREATE TRIGGER ts_insert_blocker BEFORE INSERT ON v0.events FOR EACH ROW EXECUTE FUNCTION _timescaledb_functions.insert_blocker();
ALTER TABLE ONLY _timescaledb_internal._hyper_2_1_chunk
    ADD CONSTRAINT "1_1_events_block_id_fkey" FOREIGN KEY (block_id) REFERENCES v0.blocks(id) ON UPDATE CASCADE ON DELETE CASCADE;
ALTER TABLE ONLY _timescaledb_internal._hyper_2_2_chunk
    ADD CONSTRAINT "2_3_events_block_id_fkey" FOREIGN KEY (block_id) REFERENCES v0.blocks(id) ON UPDATE CASCADE ON DELETE CASCADE;
ALTER TABLE ONLY v0.blocks
    ADD CONSTRAINT blocks_chain_id_fkey FOREIGN KEY (chain_id) REFERENCES v0.chains(id) ON UPDATE RESTRICT ON DELETE RESTRICT;
ALTER TABLE ONLY v0.events
    ADD CONSTRAINT events_block_id_fkey FOREIGN KEY (block_id) REFERENCES v0.blocks(id) ON UPDATE CASCADE ON DELETE CASCADE;
