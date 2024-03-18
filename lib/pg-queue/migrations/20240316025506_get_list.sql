CREATE OR REPLACE FUNCTION get_list(
  root_id bigint,
  max_depth int
) RETURNS TABLE(id bigint, parent bigint, item jsonb, depth integer)
  AS $$
    WITH RECURSIVE cte (id_, parent_, item_, depth_) AS
    (
      SELECT
        id, parent, item, 1 as depth
      FROM
        queue
      WHERE
        queue.id = root_id
        AND
          queue.parent IS NOT NULL
      UNION
        SELECT 
          c.parent_, t.parent, t.item, c.depth_::integer + 1
        FROM
          cte c
        JOIN queue t
        ON c.parent_ = t.id
        WHERE
          depth_ < max_depth
    )
    SELECT * FROM cte;
  $$
  LANGUAGE SQL;
