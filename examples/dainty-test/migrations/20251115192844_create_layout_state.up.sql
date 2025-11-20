-- Add up migration script here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS pg_uuidv7;

CREATE TABLE IF NOT EXISTS layout_state (
  id uuid PRIMARY KEY DEFAULT uuid_generate_v7(),
  user_id text not null default 'anonymous',
  context_key text not null,
  settings jsonb not null DEFAULT '{}',
  created_at timestamptz not null DEFAULT now (),
  updated_at timestamptz not null DEFAULT now (),
  UNIQUE (user_id, context_key)
);

CREATE INDEX IF NOT EXISTS idx_layout_state_user_context ON layout_state (user_id, context_key);
