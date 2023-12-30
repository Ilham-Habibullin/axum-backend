create table notes (
  id SERIAL PRIMARY KEY,
  text varchar(200) unique,
  user_id integer REFERENCES users(id),
  created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_DATE
)