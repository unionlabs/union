CREATE TABLE queue (
  id uuid PRIMARY KEY,
  payload_id uuid NOT NULL DEFAULT(gen_random_uuid()),
  joined timestamptz NOT NULL DEFAULT (now()),
  score integer NOT NULL
);

CREATE OR REPLACE FUNCTION min_score() RETURNS INTEGER AS $$
  SELECT COALESCE(MIN(score) - 1, 1000000) FROM queue
$$ LANGUAGE SQL;

CREATE OR REPLACE FUNCTION set_initial_score_trigger() RETURNS TRIGGER AS $$
BEGIN
  NEW.score := min_score();
  RETURN NEW;
END
$$ LANGUAGE plpgsql;

CREATE TRIGGER queue_set_initial_score BEFORE INSERT ON queue FOR EACH ROW EXECUTE FUNCTION set_initial_score_trigger();

CREATE UNIQUE INDEX idx_queue_score_id ON queue(score, id);
CREATE UNIQUE INDEX idx_queue_score ON queue(score);

ALTER TABLE queue ADD FOREIGN KEY (id) REFERENCES users(id);

CREATE TABLE contribution_status(
  id uuid PRIMARY KEY,
  started timestamptz NOT NULL DEFAULT(now()),
  expire timestamptz NOT NULL DEFAULT(now() + INTERVAL '30 minutes')
);

CREATE UNIQUE INDEX idx_contribution_status_id_expire ON contribution_status(id, expire);

ALTER TABLE contribution_status ADD FOREIGN KEY (id) REFERENCES queue(id);

CREATE TABLE contribution(
  id uuid PRIMARY KEY,
  seq SMALLSERIAL NOT NULL,
  created_at timestamptz NOT NULL DEFAULT(now()),
  success boolean
);

ALTER TABLE contribution ADD FOREIGN KEY (id) REFERENCES queue(id);

CREATE UNIQUE INDEX idx_contribution_seq ON contribution(seq);
CREATE UNIQUE INDEX idx_contribution_seq_success ON contribution(seq, success);

-- The next contributor is the one with the higest score that didn't contribute yet.
CREATE OR REPLACE FUNCTION set_next_contributor_trigger() RETURNS TRIGGER AS $$
BEGIN
  CALL set_next_contributor();
  RETURN NEW;
END
$$ LANGUAGE plpgsql;

-- Rotate the current contributor whenever a contribution is done.
CREATE TRIGGER contribution_added AFTER INSERT ON contribution FOR EACH ROW EXECUTE FUNCTION set_next_contributor_trigger();

-- Current contributor is the highest score in the queue with the contribution
-- not done yet and it's status not expired.
CREATE OR REPLACE VIEW current_contributor_id AS
  SELECT q.id
  FROM queue q
  WHERE q.score = (
    SELECT MAX(qq.score)
    FROM queue qq
    WHERE NOT EXISTS (
      SELECT * FROM contribution c WHERE c.id = qq.id
    ) AND EXISTS (
      SELECT cs.expire FROM contribution_status cs WHERE cs.id = qq.id AND cs.expire > now()
    )
 );

-- The current payload is from the latest successfull contribution
CREATE OR REPLACE VIEW current_payload_id AS
  SELECT q.payload_id
  FROM contribution c
  INNER JOIN queue q ON q.id = c.id
  WHERE c.seq = (
    SELECT MAX(cc.seq) FROM contribution cc WHERE cc.success
  );

CREATE OR REPLACE PROCEDURE set_next_contributor() AS $$
BEGIN
  IF (SELECT COUNT(*) FROM current_contributor_id) = 0 THEN
    INSERT INTO contribution_status(id) SELECT q.id FROM queue q WHERE q.score = (
      SELECT MAX(qq.score)
      FROM queue qq
      WHERE NOT EXISTS (
        SELECT * FROM contribution c WHERE c.id = qq.id
      ) AND NOT EXISTS(
        SELECT cs.expire FROM contribution_status cs WHERE cs.id = qq.id AND cs.expire < now()
      )
    );
  END IF;
END
$$ LANGUAGE plpgsql;

SELECT cron.schedule('update-contributor', '10 seconds', 'CALL set_next_contributor()');
