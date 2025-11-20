-- name: SaveLayoutState :one
INSERT INTO layout_state (id, user_id, context_key, settings, created_at, updated_at)
VALUES ($1, $2, $3, $4, now(), now())
ON CONFLICT (user_id, context_key) DO UPDATE SET
  settings = EXCLUDED.settings,
  updated_at = now()
RETURNING *;


-- name: GetLayoutState :one
SELECT *
FROM layout_state
WHERE user_id = $1 AND context_key = $2;
