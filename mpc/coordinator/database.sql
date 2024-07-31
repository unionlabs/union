BEGIN;

-- Default bucket for contributions upload
INSERT INTO storage.buckets(id, name, public) VALUES('contributions', 'contributions', false);

-----------
-- Queue --
-----------
CREATE TABLE queue (
  id uuid PRIMARY KEY,
  payload_id uuid NOT NULL DEFAULT(gen_random_uuid()),
  joined timestamptz NOT NULL DEFAULT (now()),
  score integer NOT NULL
);

ALTER TABLE queue ENABLE ROW LEVEL SECURITY;
ALTER TABLE queue ADD FOREIGN KEY (id) REFERENCES auth.users(id);
CREATE UNIQUE INDEX idx_queue_score_id ON queue(score, id);
CREATE UNIQUE INDEX idx_queue_score ON queue(score);
CREATE UNIQUE INDEX idx_queue_id_payload ON queue(id, payload_id);

CREATE POLICY view_all
  ON queue
  FOR SELECT
    TO authenticated
    USING (
      true
    );

CREATE OR REPLACE FUNCTION min_score() RETURNS INTEGER AS $$
BEGIN
  RETURN (SELECT COALESCE(MIN(score) - 1, 1000000) FROM queue);
END
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION set_initial_score_trigger() RETURNS TRIGGER AS $$
BEGIN
  NEW.score := min_score();
  RETURN NEW;
END
$$ LANGUAGE plpgsql;

CREATE TRIGGER queue_set_initial_score
BEFORE INSERT
ON queue
FOR EACH ROW
EXECUTE FUNCTION set_initial_score_trigger();

-------------------------
-- Contribution Status --
-------------------------
CREATE TABLE contribution_status(
  id uuid PRIMARY KEY,
  started timestamptz NOT NULL DEFAULT(now()),
  expire timestamptz NOT NULL DEFAULT(now() + INTERVAL '30 minutes')
);

ALTER TABLE contribution_status ENABLE ROW LEVEL SECURITY;
ALTER TABLE contribution_status ADD FOREIGN KEY (id) REFERENCES queue(id);
CREATE UNIQUE INDEX idx_contribution_status_id_expire ON contribution_status(id, expire);

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
  success boolean
);

ALTER TABLE contribution ENABLE ROW LEVEL SECURITY;
ALTER TABLE contribution ADD FOREIGN KEY (id) REFERENCES contribution_status(id);
CREATE UNIQUE INDEX idx_contribution_seq ON contribution(seq);
CREATE UNIQUE INDEX idx_contribution_seq_success ON contribution(success, seq);

CREATE POLICY view_all
  ON contribution
  FOR SELECT
    TO authenticated
    USING (
      true
    );

-- The next contributor is the one with the higest score that didn't contribute yet.
CREATE OR REPLACE FUNCTION set_next_contributor_trigger() RETURNS TRIGGER AS $$
BEGIN
  CALL set_next_contributor();
  RETURN NEW;
END
$$ LANGUAGE plpgsql;

-- Rotate the current contributor whenever a contribution is done.
CREATE TRIGGER contribution_added
AFTER INSERT
ON contribution
FOR EACH ROW
EXECUTE FUNCTION set_next_contributor_trigger();

-- Current contributor is the highest score in the queue with the contribution
-- not done yet and it's status expired without payload submitted.
CREATE OR REPLACE VIEW current_contributor_id AS
  SELECT q.id
  FROM queue q
  WHERE q.score = (
    SELECT MAX(qq.score)
    FROM queue qq
    WHERE NOT EXISTS (
      SELECT c.id FROM contribution c WHERE c.id = qq.id
    ) AND (
      EXISTS (SELECT cs.expire FROM contribution_status cs WHERE cs.id = qq.id AND cs.expire > now())
      OR
      EXISTS (SELECT cs.id FROM contribution_submitted cs WHERE cs.id = qq.id)
    )
 );

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

-- The current payload is from the latest successfull contribution
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
  IF (SELECT COUNT(*) FROM current_contributor_id) = 0 THEN
    INSERT INTO contribution_status(id) SELECT q.id FROM queue q WHERE q.score = (
      SELECT MAX(qq.score)
      FROM queue qq
      WHERE NOT EXISTS (
        SELECT c.id FROM contribution c WHERE c.id = qq.id
      ) AND NOT EXISTS (
        SELECT cs.expire FROM contribution_status cs WHERE cs.id = qq.id AND cs.expire < now()
      )
    );
  END IF;
END
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION can_upload(name varchar) RETURNS BOOLEAN AS $$
BEGIN
  RETURN (
    -- User must be the current contributor.
    (SELECT cci.id FROM current_contributor_id cci) = auth.uid()
    AND
    -- User is only allowed to submit the expected payload.
    storage.filename(name) = (SELECT q.payload_id::text FROM queue q WHERE q.id = auth.uid())
    AND
    -- Do not allow the user to interact with the file after its been submitted.
    NOT EXISTS (SELECT * FROM contribution_submitted cs WHERE cs.id = auth.uid())
  );
END
$$ LANGUAGE plpgsql;

CREATE POLICY allow_authenticated_contributor_upload_insert
  ON storage.objects
  FOR INSERT
    TO authenticated
    WITH CHECK (
      bucket_id = 'contributions'
      AND
      can_upload(name)
    );

CREATE OR REPLACE FUNCTION can_download(name varchar) RETURNS BOOLEAN AS $$
BEGIN
  RETURN (
    -- User must be the current contributor.
    (SELECT cci.id FROM current_contributor_id cci) = auth.uid()
    AND
    -- User is only allowed to download the last verified contribution.
    storage.filename(name) = (SELECT cpi.payload_id::text FROM current_payload_id cpi)
    AND
    -- Do not allow the user to interact with the file after its contribution has been submitted.
    NOT EXISTS (SELECT * FROM contribution_submitted cs WHERE cs.id = auth.uid())
  );
END
$$ LANGUAGE plpgsql;

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
  INSERT INTO contribution_submitted(id, object_id) VALUES(queue_id, object_id);
END
$$ LANGUAGE plpgsql;

-- Phase 2 contribution payload is constant size
CREATE OR REPLACE FUNCTION expected_payload_size() RETURNS INTEGER AS $$
BEGIN
  RETURN 306032532;
END
$$ LANGUAGE plpgsql;

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
      WHEN file_size = expected_payload_size()
        THEN CALL set_contribution_submitted(uuid(NEW.owner_id), NEW.id);
      ELSE
        RAISE EXCEPTION 'invalid file size, name: %, got: %, expected: %, meta: %', NEW.name, file_size, expected_payload_size(), NEW.metadata;
    END CASE;
  END IF;
  RETURN NEW;
END
$$ LANGUAGE plpgsql;

-- Rotate the current contributor whenever a contribution is done.
CREATE TRIGGER contribution_payload_uploaded
AFTER INSERT OR UPDATE
ON storage.objects
FOR EACH ROW
EXECUTE FUNCTION set_contribution_submitted_trigger();

-- Will rotate the current contributor if the slot expired without any contribution submitted
SELECT cron.schedule('update-contributor', '10 seconds', 'CALL set_next_contributor()');

COMMIT;
