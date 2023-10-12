SET check_function_bodies = false;
CREATE EXTENSION IF NOT EXISTS timescaledb WITH SCHEMA public;
COMMENT ON EXTENSION timescaledb IS 'Enables scalable inserts and complex queries for time-series data';
CREATE SCHEMA demo;
CREATE FUNCTION demo.set_current_timestamp_updated_at() RETURNS trigger
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
CREATE FUNCTION public.clone_schema(source_schema text, dest_schema text) RETURNS void
    LANGUAGE plpgsql
    AS $$
DECLARE 
  objeto text;
  buffer text;
BEGIN
    EXECUTE 'CREATE SCHEMA ' || dest_schema ;
    FOR objeto IN
        SELECT table_name::text FROM information_schema.tables WHERE table_schema = source_schema
    LOOP        
        buffer := dest_schema || '.' || objeto;
        EXECUTE 'CREATE TABLE ' || buffer || ' (LIKE ' || source_schema || '.' || objeto || ' INCLUDING CONSTRAINTS INCLUDING INDEXES INCLUDING DEFAULTS)';
        EXECUTE 'INSERT INTO ' || buffer || '(SELECT * FROM ' || source_schema || '.' || objeto || ')';
    END LOOP;
END;
$$;
CREATE FUNCTION public.set_current_timestamp_updated_at() RETURNS trigger
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
CREATE TABLE demo.faucet_claims (
    id integer NOT NULL,
    address text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    identifiers jsonb
);
CREATE SEQUENCE demo.faucet_claims_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;
ALTER SEQUENCE demo.faucet_claims_id_seq OWNED BY demo.faucet_claims.id;
CREATE TABLE demo.queue (
    id integer NOT NULL,
    item jsonb NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    length numeric GENERATED ALWAYS AS (jsonb_array_length(item)) STORED
);
CREATE SEQUENCE demo.queue_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;
ALTER SEQUENCE demo.queue_id_seq OWNED BY demo.queue.id;
CREATE TABLE demo.txes (
    id integer NOT NULL,
    data jsonb NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);
CREATE SEQUENCE demo.txes_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;
ALTER SEQUENCE demo.txes_id_seq OWNED BY demo.txes.id;
CREATE TABLE public.blocks (
    hash text NOT NULL,
    chain_id integer NOT NULL,
    height integer NOT NULL,
    is_finalized boolean DEFAULT false NOT NULL,
    extra_data jsonb,
    id integer NOT NULL,
    created_at timestamp with time zone DEFAULT now(),
    updated_at timestamp with time zone DEFAULT now()
);
CREATE SEQUENCE public.blocks_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;
ALTER SEQUENCE public.blocks_id_seq OWNED BY public.blocks.id;
CREATE TABLE public.chains (
    id integer NOT NULL,
    chain_id text NOT NULL
);
CREATE SEQUENCE public.chains_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;
ALTER SEQUENCE public.chains_id_seq OWNED BY public.chains.id;
CREATE TABLE public.events (
    index integer NOT NULL,
    data jsonb NOT NULL,
    block_id integer NOT NULL,
    id integer NOT NULL
);
CREATE SEQUENCE public.events_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;
ALTER SEQUENCE public.events_id_seq OWNED BY public.events.id;
ALTER TABLE ONLY demo.faucet_claims ALTER COLUMN id SET DEFAULT nextval('demo.faucet_claims_id_seq'::regclass);
ALTER TABLE ONLY demo.queue ALTER COLUMN id SET DEFAULT nextval('demo.queue_id_seq'::regclass);
ALTER TABLE ONLY demo.txes ALTER COLUMN id SET DEFAULT nextval('demo.txes_id_seq'::regclass);
ALTER TABLE ONLY public.blocks ALTER COLUMN id SET DEFAULT nextval('public.blocks_id_seq'::regclass);
ALTER TABLE ONLY public.chains ALTER COLUMN id SET DEFAULT nextval('public.chains_id_seq'::regclass);
ALTER TABLE ONLY public.events ALTER COLUMN id SET DEFAULT nextval('public.events_id_seq'::regclass);
ALTER TABLE ONLY demo.faucet_claims
    ADD CONSTRAINT faucet_claims_pkey PRIMARY KEY (id);
ALTER TABLE ONLY demo.queue
    ADD CONSTRAINT queue_pkey PRIMARY KEY (id);
ALTER TABLE ONLY demo.txes
    ADD CONSTRAINT txes_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.blocks
    ADD CONSTRAINT blocks_hash_chain_id_key UNIQUE (hash, chain_id);
ALTER TABLE ONLY public.blocks
    ADD CONSTRAINT blocks_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.chains
    ADD CONSTRAINT chains_id_key UNIQUE (id);
ALTER TABLE ONLY public.chains
    ADD CONSTRAINT chains_pkey PRIMARY KEY (id, chain_id);
ALTER TABLE ONLY public.events
    ADD CONSTRAINT events_block_id_index_key UNIQUE (block_id, index);
ALTER TABLE ONLY public.events
    ADD CONSTRAINT events_pkey PRIMARY KEY (id);
CREATE INDEX idx_chains_chain_id ON public.chains USING btree (chain_id);
CREATE TRIGGER set_demo_faucet_claims_updated_at BEFORE UPDATE ON demo.faucet_claims FOR EACH ROW EXECUTE FUNCTION demo.set_current_timestamp_updated_at();
COMMENT ON TRIGGER set_demo_faucet_claims_updated_at ON demo.faucet_claims IS 'trigger to set value of column "updated_at" to current timestamp on row update';
CREATE TRIGGER set_demo_queue_updated_at BEFORE UPDATE ON demo.queue FOR EACH ROW EXECUTE FUNCTION demo.set_current_timestamp_updated_at();
COMMENT ON TRIGGER set_demo_queue_updated_at ON demo.queue IS 'trigger to set value of column "updated_at" to current timestamp on row update';
CREATE TRIGGER set_demo_txes_updated_at BEFORE UPDATE ON demo.txes FOR EACH ROW EXECUTE FUNCTION demo.set_current_timestamp_updated_at();
COMMENT ON TRIGGER set_demo_txes_updated_at ON demo.txes IS 'trigger to set value of column "updated_at" to current timestamp on row update';
CREATE TRIGGER set_public_blocks_updated_at BEFORE UPDATE ON public.blocks FOR EACH ROW EXECUTE FUNCTION public.set_current_timestamp_updated_at();
COMMENT ON TRIGGER set_public_blocks_updated_at ON public.blocks IS 'trigger to set value of column "updated_at" to current timestamp on row update';
ALTER TABLE ONLY public.blocks
    ADD CONSTRAINT blocks_chain_id_fkey FOREIGN KEY (chain_id) REFERENCES public.chains(id) ON UPDATE CASCADE ON DELETE CASCADE;
ALTER TABLE ONLY public.events
    ADD CONSTRAINT events_block_id_fkey FOREIGN KEY (block_id) REFERENCES public.blocks(id) ON UPDATE CASCADE ON DELETE CASCADE;
