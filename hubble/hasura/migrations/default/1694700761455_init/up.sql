SET check_function_bodies = false;
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
CREATE TABLE public.addresses (
    address text NOT NULL,
    chain_id integer NOT NULL,
    id integer NOT NULL
);
CREATE SEQUENCE public.addresses_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;
ALTER SEQUENCE public.addresses_id_seq OWNED BY public.addresses.id;
CREATE TABLE public.blocks (
    hash text NOT NULL,
    chain_id integer NOT NULL,
    height integer NOT NULL,
    id integer NOT NULL,
    is_finalized boolean DEFAULT false,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now(),
    extra_data jsonb
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
    id integer NOT NULL,
    index integer NOT NULL,
    data jsonb NOT NULL,
    block_id integer NOT NULL
);
CREATE VIEW public.coin_receiveds AS
 SELECT events.id,
    (((events.data -> 'attributes'::text) -> 0) ->> 'value'::text) AS receiver,
    "substring"((((events.data -> 'attributes'::text) -> 1) ->> 'value'::text), '^\d+'::text) AS amount,
    "substring"((((events.data -> 'attributes'::text) -> 1) ->> 'value'::text), '[a-zA-Z].*'::text) AS denom
   FROM public.events
  WHERE ((events.data ->> 'type'::text) = 'coin_received'::text);
CREATE VIEW public.coin_spents AS
 SELECT events.id,
    (((events.data -> 'attributes'::text) -> 0) ->> 'value'::text) AS spender,
    "substring"((((events.data -> 'attributes'::text) -> 1) ->> 'value'::text), '^\d+'::text) AS amount,
    "substring"((((events.data -> 'attributes'::text) -> 1) ->> 'value'::text), '[a-zA-Z].*'::text) AS denom
   FROM public.events
  WHERE ((events.data ->> 'type'::text) = 'coin_spent'::text);
CREATE VIEW public.coinbases AS
 SELECT events.id,
    (((events.data -> 'attributes'::text) -> 0) ->> 'value'::text) AS minter,
    "substring"((((events.data -> 'attributes'::text) -> 1) ->> 'value'::text), '^\d+'::text) AS amount,
    "substring"((((events.data -> 'attributes'::text) -> 1) ->> 'value'::text), '[a-zA-Z].*'::text) AS denom
   FROM public.events
  WHERE ((events.data ->> 'type'::text) = 'coinbase'::text);
CREATE VIEW public.commissions AS
 SELECT events.id,
    (((events.data -> 'attributes'::text) -> 1) ->> 'value'::text) AS validator,
    "substring"((((events.data -> 'attributes'::text) -> 0) ->> 'value'::text), '^\d+'::text) AS amount,
    "substring"((((events.data -> 'attributes'::text) -> 0) ->> 'value'::text), '[a-zA-Z].*'::text) AS denom
   FROM public.events
  WHERE ((events.data ->> 'type'::text) = 'commission'::text);
CREATE SEQUENCE public.events_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;
ALTER SEQUENCE public.events_id_seq OWNED BY public.events.id;
CREATE VIEW public.messages AS
 SELECT events.id,
    (((events.data -> 'attributes'::text) -> 0) ->> 'value'::text) AS sender
   FROM public.events
  WHERE ((events.data ->> 'type'::text) = 'message'::text);
CREATE VIEW public.mints AS
 SELECT events.id,
    (((events.data -> 'attributes'::text) -> 0) ->> 'value'::text) AS bonded_ratio,
    (((events.data -> 'attributes'::text) -> 1) ->> 'value'::text) AS inflation,
    (((events.data -> 'attributes'::text) -> 2) ->> 'value'::text) AS annual_provisions,
    (((events.data -> 'attributes'::text) -> 3) ->> 'value'::text) AS amount
   FROM public.events
  WHERE ((events.data ->> 'type'::text) = 'mint'::text);
CREATE VIEW public.rewards AS
 SELECT events.id,
    (((events.data -> 'attributes'::text) -> 1) ->> 'value'::text) AS validator,
    "substring"((((events.data -> 'attributes'::text) -> 0) ->> 'value'::text), '^\d+'::text) AS amount,
    "substring"((((events.data -> 'attributes'::text) -> 0) ->> 'value'::text), '[a-zA-Z].*'::text) AS denom
   FROM public.events
  WHERE ((events.data ->> 'type'::text) = 'rewards'::text);
CREATE VIEW public.transfers AS
 SELECT events.id,
    (((events.data -> 'attributes'::text) -> 0) ->> 'value'::text) AS recipient,
    (((events.data -> 'attributes'::text) -> 1) ->> 'value'::text) AS sender,
    "substring"((((events.data -> 'attributes'::text) -> 2) ->> 'value'::text), '^\d+'::text) AS amount,
    "substring"((((events.data -> 'attributes'::text) -> 2) ->> 'value'::text), '[a-zA-Z].*'::text) AS denom
   FROM public.events
  WHERE ((events.data ->> 'type'::text) = 'transfer'::text);
ALTER TABLE ONLY public.addresses ALTER COLUMN id SET DEFAULT nextval('public.addresses_id_seq'::regclass);
ALTER TABLE ONLY public.blocks ALTER COLUMN id SET DEFAULT nextval('public.blocks_id_seq'::regclass);
ALTER TABLE ONLY public.chains ALTER COLUMN id SET DEFAULT nextval('public.chains_id_seq'::regclass);
ALTER TABLE ONLY public.events ALTER COLUMN id SET DEFAULT nextval('public.events_id_seq'::regclass);
ALTER TABLE ONLY public.addresses
    ADD CONSTRAINT addresses_address_chain_id_key UNIQUE (address, chain_id);
ALTER TABLE ONLY public.addresses
    ADD CONSTRAINT addresses_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.blocks
    ADD CONSTRAINT blocks_pkey PRIMARY KEY (id);
ALTER TABLE ONLY public.chains
    ADD CONSTRAINT chains_id_key UNIQUE (id);
ALTER TABLE ONLY public.chains
    ADD CONSTRAINT chains_pkey PRIMARY KEY (id, chain_id);
ALTER TABLE ONLY public.events
    ADD CONSTRAINT events_pkey PRIMARY KEY (id);
CREATE INDEX coin_received_idx ON public.events USING btree (((data ->> 'type'::text)));
CREATE INDEX idx_blocks_chain_id ON public.blocks USING btree (chain_id);
CREATE INDEX idx_blocks_chain_id_height ON public.blocks USING btree (chain_id, height);
CREATE INDEX idx_blocks_height_desc ON public.blocks USING btree (height DESC);
CREATE INDEX idx_chains_chain_id ON public.chains USING btree (chain_id);
CREATE INDEX idx_events_block_id ON public.events USING btree (block_id);
CREATE INDEX receiver ON public.events USING gin (((((data -> 'attributes'::text) -> 0) -> 'value'::text))) WHERE ((data ->> 'type'::text) = 'coin_received'::text);
CREATE TRIGGER set_public_blocks_updated_at BEFORE UPDATE ON public.blocks FOR EACH ROW EXECUTE FUNCTION public.set_current_timestamp_updated_at();
COMMENT ON TRIGGER set_public_blocks_updated_at ON public.blocks IS 'trigger to set value of column "updated_at" to current timestamp on row update';
ALTER TABLE ONLY public.blocks
    ADD CONSTRAINT blocks_chain_id_fkey FOREIGN KEY (chain_id) REFERENCES public.chains(id) ON UPDATE CASCADE ON DELETE CASCADE;
ALTER TABLE ONLY public.events
    ADD CONSTRAINT events_block_id_fkey FOREIGN KEY (block_id) REFERENCES public.blocks(id) ON UPDATE CASCADE ON DELETE CASCADE;
