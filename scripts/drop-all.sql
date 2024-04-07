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
