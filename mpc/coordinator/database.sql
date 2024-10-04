BEGIN;

-----------
-- Erase --
-----------
TRUNCATE TABLE public.log;
TRUNCATE TABLE auth.users CASCADE;
DELETE FROM storage.objects o
WHERE o.bucket_id = 'contributions'
AND o.name <> '00000000-0000-0000-0000-000000000000';

-- Default bucket for contributions upload
INSERT INTO storage.buckets(id, name, public) VALUES('contributions', 'contributions', false);

CREATE TABLE wallet_address(
  id uuid PRIMARY KEY,
  wallet text NOT NULL
);

ALTER TABLE wallet_address ENABLE ROW LEVEL SECURITY;
ALTER TABLE wallet_address ADD FOREIGN KEY (id) REFERENCES auth.users(id);

CREATE POLICY view_self
  ON wallet_address
  FOR SELECT
    TO authenticated
    USING (
      (SELECT auth.uid()) = id
    );

CREATE POLICY allow_insert_self
  ON wallet_address
  FOR INSERT
    TO authenticated
    WITH CHECK (
      (SELECT auth.uid()) = id
    );

CREATE TABLE waitlist(
  id uuid PRIMARY KEY,
  created_at timestamptz NOT NULL DEFAULT(now()),
  seq smallserial NOT NULL,
);

ALTER TABLE waitlist ENABLE ROW LEVEL SECURITY;
ALTER TABLE waitlist ADD FOREIGN KEY (id) REFERENCES auth.users(id);
CREATE UNIQUE INDEX idx_waitlist_seq ON waitlist(seq);

CREATE POLICY view_self
  ON waitlist
  FOR SELECT
    TO authenticated
    USING (
      (SELECT auth.uid()) = id
    );

CREATE POLICY allow_insert_self
  ON waitlist
  FOR INSERT
    TO authenticated
    WITH CHECK (
      (SELECT auth.uid()) = id AND open_to_public() = false
    );

CREATE OR REPLACE FUNCTION waitlist_overwrite_timestamp() RETURNS TRIGGER AS $$
BEGIN
  NEW.created_at = now();
  RETURN NEW;
END
$$ LANGUAGE plpgsql SET search_path = '';

CREATE TRIGGER waitlist_overwrite_timestamp
BEFORE INSERT
ON waitlist
FOR EACH ROW
EXECUTE FUNCTION waitlist_overwrite_timestamp();

-----------
-- Queue --
-----------
CREATE OR REPLACE FUNCTION open_to_public() RETURNS boolean AS $$
BEGIN
  RETURN false;
END
$$ LANGUAGE plpgsql SET search_path = '';

CREATE TABLE queue (
  id uuid PRIMARY KEY,
  payload_id uuid NOT NULL DEFAULT(gen_random_uuid()),
  joined timestamptz NOT NULL DEFAULT (now()),
  score integer NOT NULL
);

ALTER TABLE queue ENABLE ROW LEVEL SECURITY;
ALTER TABLE queue ADD FOREIGN KEY (id) REFERENCES auth.users(id);
CREATE UNIQUE INDEX idx_queue_score_id ON queue(score, id);
CREATE UNIQUE INDEX idx_queue_id_payload ON queue(id, payload_id);
CREATE INDEX idx_queue_score ON queue(score);

CREATE POLICY view_all
  ON queue
  FOR SELECT
    TO authenticated
    USING (
      true
    );

CREATE OR REPLACE VIEW current_queue AS
  (
    SELECT *, (SELECT COUNT(*) FROM queue qq
                WHERE
                NOT EXISTS (SELECT cs.id FROM contribution_status cs WHERE cs.id = qq.id)
                AND qq.score > q.score
    ) + 1 AS position FROM queue q
    WHERE
    -- Contribution round not started
    NOT EXISTS (SELECT cs.id FROM contribution_status cs WHERE cs.id = q.id)
    ORDER BY q.score DESC
  );

ALTER VIEW current_queue SET (security_invoker = on);

CREATE OR REPLACE FUNCTION min_score() RETURNS INTEGER AS $$
BEGIN
  RETURN (SELECT COALESCE(MIN(score) - 1, 1000000) FROM public.queue);
END
$$ LANGUAGE plpgsql SET search_path = '';

CREATE OR REPLACE FUNCTION set_initial_score_trigger() RETURNS TRIGGER AS $$
BEGIN
  NEW.score := public.min_score();
  RETURN NEW;
END
$$ LANGUAGE plpgsql SET search_path = '';

CREATE TRIGGER queue_set_initial_score
BEFORE INSERT
ON queue
FOR EACH ROW
EXECUTE FUNCTION set_initial_score_trigger();

-----------
--  Code --
-----------
CREATE TABLE code (
  id text PRIMARY KEY,
  user_id uuid DEFAULT NULL,
  display_name text NOT NULL DEFAULT ('John Doe')
);

ALTER TABLE code ENABLE ROW LEVEL SECURITY;
ALTER TABLE code ADD FOREIGN KEY (user_id) REFERENCES auth.users(id);
CREATE UNIQUE INDEX idx_code_user_id ON code(user_id);

CREATE OR REPLACE FUNCTION rejoin_queue() RETURNS void AS $$
BEGIN
  IF (NOT EXISTS (SELECT * FROM public.queue q WHERE q.id = (SELECT auth.uid()))) THEN
    RAISE EXCEPTION 'not_in_queue';
  END IF;
  IF (EXISTS (SELECT * FROM public.contribution_submitted cs WHERE cs.id = (SELECT auth.uid()))) THEN
    RAISE EXCEPTION 'already_submitted';
  END IF;
  IF (NOT EXISTS (SELECT * FROM public.contribution_status cs WHERE cs.id = (SELECT auth.uid()) AND cs.expire < now())) THEN
    RAISE EXCEPTION 'not_expired';
  END IF;
  UPDATE public.queue SET score = public.min_score() WHERE id = (SELECT auth.uid());
  DELETE FROM public.contribution_status cs WHERE id = (SELECT auth.uid());
END
$$ LANGUAGE plpgsql SECURITY DEFINER SET search_path = '';

CREATE OR REPLACE FUNCTION redeem(code_id text) RETURNS void AS $$
DECLARE
  redeemed_code public.code%ROWTYPE := NULL;
BEGIN
  UPDATE public.code c
    SET user_id = (SELECT auth.uid())
    WHERE c.id = encode(sha256(code_id::bytea), 'hex')
    AND c.user_id IS NULL
    RETURNING * INTO redeemed_code;
  IF (redeemed_code IS NULL) THEN
    RAISE EXCEPTION 'redeem_code_invalid';
  END IF;
  INSERT INTO public.queue(id) VALUES ((SELECT auth.uid()));
  PERFORM public.do_log(
    json_build_object(
      'type', 'redeem',
      'user', (SELECT un.user_name FROM public.user_name un WHERE un.id = (SELECT auth.uid()))
    )
  );
  PERFORM public.do_log(
    json_build_object(
      'type', 'join_queue',
      'user', (SELECT un.user_name FROM public.user_name un WHERE un.id = (SELECT auth.uid()))
    )
  );
END
$$ LANGUAGE plpgsql SECURITY DEFINER SET search_path = '';

CREATE OR REPLACE FUNCTION join_queue(code_id text) RETURNS void AS $$
BEGIN
  IF (code_id IS NULL) THEN
    IF (public.open_to_public()) THEN
      INSERT INTO public.queue(id) VALUES ((SELECT auth.uid()));
      PERFORM public.do_log(
        json_build_object(
          'type', 'join_queue',
          'user', (SELECT un.user_name FROM public.user_name un WHERE un.id = (SELECT auth.uid()))
        )
      );
    ELSE
      INSERT INTO public.waitlist(id) VALUES ((SELECT auth.uid()));
      PERFORM public.do_log(
        json_build_object(
          'type', 'join_waitlist',
          'user', (SELECT un.user_name FROM public.user_name un WHERE un.id = (SELECT auth.uid()))
        )
      );
    END IF;
  ELSE
    PERFORM public.redeem(code_id);
  END IF;
END
$$ LANGUAGE plpgsql SECURITY DEFINER SET search_path = '';

-- Username
CREATE OR REPLACE VIEW user_name AS
  SELECT u.id, ('anon_' || left(encode(sha256(u.id::text::bytea), 'hex'), 20)) AS user_name FROM auth.users u;

ALTER VIEW user_name SET (security_invoker = off);

-- CREATE OR REPLACE VIEW user_name AS
--   SELECT u.id,
--     COALESCE(
--       (SELECT c.display_name FROM public.code c WHERE c.user_id = u.id),
--       COALESCE(
--         u.raw_user_meta_data->>'user_name',
--         u.raw_user_meta_data->>'name'
--       )
--     ) AS user_name FROM auth.users u;

-- ALTER VIEW user_name SET (security_invoker = off);

-------------------------
-- Contribution Status --
-------------------------
CREATE OR REPLACE FUNCTION expiration_delay() RETURNS INTERVAL AS $$
BEGIN
  RETURN INTERVAL '1 hour';
END
$$ LANGUAGE plpgsql SET search_path = '';

CREATE TABLE contribution_status(
  id uuid PRIMARY KEY,
  started timestamptz NOT NULL DEFAULT(now()),
  expire timestamptz NOT NULL DEFAULT(now() + expiration_delay())
);

ALTER TABLE contribution_status ENABLE ROW LEVEL SECURITY;
ALTER TABLE contribution_status ADD FOREIGN KEY (id) REFERENCES queue(id);
CREATE UNIQUE INDEX idx_contribution_status_id_expire ON contribution_status(id, expire);
CREATE UNIQUE INDEX idx_contribution_status_id_started ON contribution_status(id, started);

CREATE POLICY view_all
  ON contribution_status
  FOR SELECT
    TO authenticated
    USING (
      true
    );

----------------------------
-- Contribution Submitted --
----------------------------
CREATE TABLE contribution_submitted(
  id uuid PRIMARY KEY,
  object_id uuid NOT NULL,
  created_at timestamptz NOT NULL DEFAULT(now())
);

ALTER TABLE contribution_submitted ENABLE ROW LEVEL SECURITY;
ALTER TABLE contribution_submitted ADD FOREIGN KEY (id) REFERENCES contribution_status(id);
ALTER TABLE contribution_submitted ADD FOREIGN KEY (object_id) REFERENCES storage.objects(id);

CREATE INDEX idx_contribution_submitted_object ON contribution_submitted(object_id);
CREATE UNIQUE INDEX idx_contribution_submitted_id_created_at ON contribution_submitted(id, created_at);

CREATE POLICY view_all
  ON contribution_submitted
  FOR SELECT
    TO authenticated
    USING (
      true
    );

------------------
-- Contribution --
------------------
CREATE TABLE contribution(
  id uuid PRIMARY KEY,
  seq smallserial NOT NULL,
  created_at timestamptz NOT NULL DEFAULT(now()),
  success boolean NOT NULL
);

ALTER TABLE contribution ENABLE ROW LEVEL SECURITY;
ALTER TABLE contribution ADD FOREIGN KEY (id) REFERENCES contribution_submitted(id);
CREATE UNIQUE INDEX idx_contribution_seq ON contribution(seq);
CREATE UNIQUE INDEX idx_contribution_seq_success ON contribution(success, seq);
CREATE UNIQUE INDEX idx_contribution_id_created_at ON contribution(id, created_at);

CREATE POLICY view_all
  ON contribution
  FOR SELECT
    TO authenticated
    USING (
      true
    );

-- The next contributor is the one with the highest score that didn't contribute yet.
CREATE OR REPLACE FUNCTION set_next_contributor_trigger() RETURNS TRIGGER AS $$
BEGIN
  PERFORM public.do_log(
      json_build_object(
        'type', 'contribution_verified',
        'user', (SELECT un.user_name FROM public.user_name un WHERE un.id = NEW.id),
        'success', NEW.success
      )
  );
  CALL public.set_next_contributor();
  RETURN NEW;
END
$$ LANGUAGE plpgsql SET search_path = '';

-- Rotate the current contributor whenever a contribution is done.
CREATE TRIGGER contribution_added
AFTER INSERT
ON contribution
FOR EACH ROW
EXECUTE FUNCTION set_next_contributor_trigger();

CREATE OR REPLACE VIEW current_verification_average AS (
  SELECT AVG(c.created_at - cs.created_at) AS verification_average
  FROM contribution c
  INNER JOIN contribution_submitted cs ON (c.id = cs.id)
);

ALTER VIEW current_verification_average SET (security_invoker = on);

CREATE OR REPLACE VIEW current_contribution_average AS (
  SELECT AVG(cs.created_at - c.started) AS contribution_average
  FROM contribution_status c
  INNER JOIN contribution_submitted cs ON (c.id = cs.id)
);

ALTER VIEW current_contribution_average SET (security_invoker = on);

-- Current contributor is the highest score in the queue with the contribution
-- not done yet and it's status expired without payload submitted.
CREATE OR REPLACE VIEW current_contributor_id AS
  SELECT qq.id
  FROM queue qq
  WHERE NOT EXISTS (
    SELECT c.id FROM contribution c WHERE c.id = qq.id
  ) AND (
    EXISTS (SELECT cs.expire FROM contribution_status cs WHERE cs.id = qq.id AND cs.expire > now())
    OR
    EXISTS (SELECT cs.id FROM contribution_submitted cs WHERE cs.id = qq.id)
  )
  ORDER BY qq.score DESC
  LIMIT 1;

ALTER VIEW current_contributor_id SET (security_invoker = on);

-- Materialized ?
CREATE OR REPLACE VIEW current_queue_position AS
  SELECT
  CASE WHEN (SELECT cci.id FROM current_contributor_id cci) = auth.uid() THEN
      0
  ELSE
      (
        SELECT COUNT(*) + 1
        FROM queue q
        WHERE
        -- Better score
        q.score > (SELECT qq.score FROM queue qq WHERE qq.id = auth.uid())
        AND
        -- Contribution round not started
        NOT EXISTS (SELECT cs.id FROM contribution_status cs WHERE cs.id = q.id)
      )
  END AS position;

ALTER VIEW current_queue_position SET (security_invoker = on);

-- The current payload is from the latest successful contribution
CREATE OR REPLACE VIEW current_payload_id AS
  SELECT COALESCE(
    (SELECT q.payload_id
     FROM contribution c
     INNER JOIN queue q USING(id)
     WHERE c.seq = (
       SELECT MAX(cc.seq) FROM contribution cc WHERE cc.success
     )
    ),
    uuid_nil()
  ) AS payload_id;

ALTER VIEW current_payload_id SET (security_invoker = on);

CREATE OR REPLACE PROCEDURE set_next_contributor() AS $$
BEGIN
  IF (NOT EXISTS (SELECT cci.id FROM public.current_contributor_id cci)) THEN
    INSERT INTO public.contribution_status(id)
    SELECT cq.id
    FROM public.current_queue cq
    LIMIT 1;
    IF (EXISTS (SELECT cci.id FROM public.current_contributor_id cci)) THEN
      PERFORM public.do_log(
        json_build_object(
          'type', 'contribution_started',
          'user', (SELECT un.user_name FROM public.user_name un WHERE un.id = (SELECT cci.id FROM public.current_contributor_id cci))
        )
      );
      IF (EXISTS (SELECT cq.id FROM public.current_queue cq WHERE cq.position = 5)) THEN
        -- I know it's ugly, just for the alert email
        -- The JWT here is the public anon one, already embedded in the frontend
        -- 5th in the queue get alerted
        PERFORM net.http_post(
            url := 'https://otfaamdxmgnkjqsosxye.supabase.co/functions/v1/ping',
            headers := '{"Content-Type": "application/json", "Authorization": "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6Im90ZmFhbWR4bWdua2pxc29zeHllIiwicm9sZSI6ImFub24iLCJpYXQiOjE3MjEzMjA5NDMsImV4cCI6MjAzNjg5Njk0M30.q91NJPFFHKJXnbhbpUYwsB0NmimtD7pGPx6PkbB_A3w"}'::jsonb,
            body := concat(
              '{"email": "',
              (SELECT u.email
               FROM auth.users u
               WHERE u.id = (
                 SELECT cq.id
                 FROM public.current_queue cq
                 WHERE cq.position = 5
               )),
               '", "secret":"',
               (SELECT private.ping_secret()),
               '"}'
            )::jsonb
        );
      END IF;
    END IF;
  END IF;
END
$$ LANGUAGE plpgsql SECURITY DEFINER SET search_path = '';

CREATE OR REPLACE FUNCTION can_upload(name varchar) RETURNS BOOLEAN AS $$
BEGIN
  RETURN (
    -- User must be the current contributor.
    (SELECT cci.id FROM public.current_contributor_id cci) = auth.uid()
    AND
    -- User is only allowed to submit the expected payload.
    storage.filename(name) = (SELECT q.payload_id::text FROM public.queue q WHERE q.id = auth.uid())
    AND
    -- Do not allow the user to interact with the file after its been submitted.
    NOT EXISTS (SELECT * FROM public.contribution_submitted cs WHERE cs.id = auth.uid())
  );
END
$$ LANGUAGE plpgsql SET search_path = '';

CREATE POLICY allow_authenticated_contributor_upload_insert
  ON storage.objects
  FOR INSERT
    TO authenticated
    WITH CHECK (
      bucket_id = 'contributions'
      AND
      can_upload(name)
    );

CREATE POLICY allow_service_insert
  ON storage.objects
  FOR INSERT
    TO service_role
    WITH CHECK (
      true
    );

CREATE OR REPLACE FUNCTION can_download(name varchar) RETURNS BOOLEAN AS $$
BEGIN
  RETURN (
    -- User must be the current contributor.
    (SELECT cci.id FROM public.current_contributor_id cci) = auth.uid()
    AND
    -- User is only allowed to download the last verified contribution.
    storage.filename(name) = (SELECT cpi.payload_id::text FROM public.current_payload_id cpi)
    AND
    -- Do not allow the user to interact with the file after its contribution has been submitted.
    NOT EXISTS (SELECT * FROM public.contribution_submitted cs WHERE cs.id = auth.uid())
  );
END
$$ LANGUAGE plpgsql SET search_path = '';

CREATE POLICY allow_authenticated_contributor_download
  ON storage.objects
  FOR SELECT
    TO authenticated
    USING (
      bucket_id = 'contributions'
      AND
      can_download(name)
    );

CREATE OR REPLACE PROCEDURE set_contribution_submitted(queue_id uuid, object_id uuid) AS $$
BEGIN
  INSERT INTO public.contribution_submitted(id, object_id) VALUES(queue_id, object_id);
  PERFORM public.do_log(
    json_build_object(
      'type', 'contribution_submitted',
      'user', (SELECT un.user_name FROM public.user_name un WHERE un.id = queue_id)
    )
  );
END
$$ LANGUAGE plpgsql SET search_path = '';

-- Phase 2 contribution payload is constant size
CREATE OR REPLACE FUNCTION expected_payload_size() RETURNS INTEGER AS $$
BEGIN
  RETURN 306032532;
END
$$ LANGUAGE plpgsql SET search_path = '';

-- Metadata pushed on upload.
-- {
--   "eTag": "\"c019643e056d8d687086c1e125f66ad8-1\"",
--   "size": 1000,
--   "mimetype": "binary/octet-stream",
--   "cacheControl": "no-cache",
--   "lastModified": "2024-07-27T23:03:32.000Z",
--   "contentLength": 1000,
--   "httpStatusCode": 200
--   }
CREATE OR REPLACE FUNCTION set_contribution_submitted_trigger() RETURNS TRIGGER AS $$
DECLARE
  file_size integer;
BEGIN
  -- For some reason, supa pushes placeholder files.
  IF (NEW.metadata IS NOT NULL) THEN
    file_size := (NEW.metadata->>'size')::integer;
    CASE
      WHEN file_size = public.expected_payload_size()
        THEN CALL public.set_contribution_submitted(uuid(NEW.owner_id), NEW.id);
      ELSE
        RAISE EXCEPTION 'invalid file size, name: %, got: %, expected: %, meta: %', NEW.name, file_size, expected_payload_size(), NEW.metadata;
    END CASE;
  END IF;
  RETURN NEW;
END
$$ LANGUAGE plpgsql SET search_path = '';

-- Rotate the current contributor whenever a contribution is done.
CREATE TRIGGER contribution_payload_uploaded
AFTER INSERT OR UPDATE
ON storage.objects
FOR EACH ROW
EXECUTE FUNCTION set_contribution_submitted_trigger();

-----------------
-- Attestation --
-----------------
CREATE TABLE contribution_signature(
  id uuid PRIMARY KEY,
  public_key text NOT NULL,
  signature text NOT NULL,
  public_key_hash text GENERATED ALWAYS AS (encode(sha256(decode(public_key, 'hex')), 'hex')) STORED
);

ALTER TABLE contribution_signature ENABLE ROW LEVEL SECURITY;
ALTER TABLE contribution_signature ADD FOREIGN KEY (id) REFERENCES contribution_status(id);
CREATE UNIQUE INDEX idx_contribution_signature_pkh ON contribution_signature(public_key_hash);

CREATE POLICY view_all
  ON contribution_signature
  FOR SELECT
    TO authenticated
    USING (
      true
    );

CREATE POLICY allow_insert_self
  ON contribution_signature
  FOR INSERT
    TO authenticated
    WITH CHECK (
      (SELECT auth.uid()) = id
    );

CREATE OR REPLACE VIEW current_user_state AS (
  SELECT
    (EXISTS (SELECT * FROM public.waitlist WHERE id = (SELECT auth.uid()))) AS in_waitlist,
    (EXISTS (SELECT * FROM public.code WHERE user_id = (SELECT auth.uid()))) AS has_redeemed,
    (EXISTS (SELECT * FROM public.queue WHERE id = (SELECT auth.uid()))) AS in_queue,
    (SELECT un.user_name FROM public.user_name un WHERE un.id = (SELECT auth.uid())) AS display_name,
    ((SELECT COUNT(*)
      FROM public.waitlist w
      WHERE w.id <> (SELECT auth.uid())
      AND w.seq < (SELECT ww.seq FROM public.waitlist ww WHERE ww.id = (SELECT auth.uid()))
    ) + 1) AS waitlist_position
);

ALTER VIEW current_user_state SET (security_invoker = off);

-----------------
-- Logging     --
-----------------
CREATE TABLE log(
  id smallserial PRIMARY KEY,
  created_at timestamptz NOT NULL DEFAULT(now()),
  message jsonb NOT NULL
);

ALTER TABLE log ENABLE ROW LEVEL SECURITY;

CREATE POLICY view_all
  ON log
  FOR SELECT
    TO authenticated
    USING (
      true
    );

CREATE OR REPLACE FUNCTION do_log(message json) RETURNS void AS $$
BEGIN
  INSERT INTO public.log(message) VALUES (message);
END
$$ LANGUAGE plpgsql SET search_path = '';

CREATE MATERIALIZED VIEW IF NOT EXISTS users_contribution AS (
  SELECT c.id,
         un.user_name,
         u.raw_user_meta_data->>'avatar_url' AS avatar_url,
         c.seq,
         q.payload_id,
         cs.public_key,
         cs.signature,
         cs.public_key_hash,
         s.started AS time_started,
         su.created_at AS time_submitted,
         c.created_at AS time_verified,
         w.wallet AS wallet,
         (su.created_at - s.started) AS time_contribute
  FROM public.contribution c
  INNER JOIN public.queue q ON (c.id = q.id)
  INNER JOIN public.contribution_status s ON (c.id = s.id)
  INNER JOIN public.contribution_submitted su ON (c.id = su.id)
  INNER JOIN public.contribution_signature cs ON (c.id = cs.id)
  INNER JOIN public.wallet_address w ON (c.id = w.id)
  INNER JOIN public.user_name un ON (c.id = un.id)
  INNER JOIN auth.users u ON (c.id = u.id)
  WHERE c.success
  ORDER BY c.seq ASC
);

CREATE UNIQUE INDEX idx_users_contribution_user_id ON users_contribution(id);
CREATE UNIQUE INDEX idx_users_contribution_pkh ON users_contribution(public_key_hash);

----------
-- CRON --
----------

-- Will rotate the current contributor if the slot expired without any contribution submitted
SELECT cron.schedule('update-contributor', '10 seconds', 'CALL set_next_contributor()');

SELECT cron.schedule('update-users-contribution', '30 seconds', 'REFRESH MATERIALIZED VIEW CONCURRENTLY public.users_contribution');

COMMIT;
