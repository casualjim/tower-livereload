-- Add down migration script here
DROP INDEX IF EXISTS idx_layout_state_user_context;
DROP TABLE IF EXISTS layout_state;
