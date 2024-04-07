#!/bin/bash

NAME="${CONTAINER_NAME:=dev-postgres-erp}"
DB_USER="${POSTGRES_USER:=postgres}"
DB_PASSWORD="${POSTGRES_PASSWORD:=postgrespw}"
DB_NAME="${POSTGRES_DB:=erp}"
DB_PORT="${POSTGRES_PORT:=5432}"
DB_HOST="${POSTGRES_HOST:=localhost}"

export PGPASSWORD="${DB_PASSWORD}"

sql_script=$(
	cat <<'EOM'
DO $$
DECLARE t record;
DECLARE r record;
BEGIN
  -- Drop tables (order matters to avoid dependency issues)
  FOR r IN (SELECT tablename FROM pg_tables WHERE schemaname = 'public') LOOP
    EXECUTE 'DROP TABLE IF EXISTS ' || quote_ident(r.tablename) || ' CASCADE';
  END LOOP;

  -- Drop user-defined types
  FOR t IN (SELECT typname FROM pg_type WHERE typnamespace = '2200') LOOP
    EXECUTE 'DROP TYPE IF EXISTS ' || quote_ident(t.typname) || ' CASCADE';
  END LOOP;
END $$;
EOM
)
docker exec -it "${NAME}" psql -U "${DB_USER}" -h "${DB_HOST}" -p "${DB_PORT}" -d "${DB_NAME}" -c "${sql_script}"
