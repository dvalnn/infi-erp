CREATE DOMAIN tool_type AS char(2)
CHECK (VALUE IN ('T1', 'T2', 'T3', 'T4', 'T5', 'T6'));

CREATE TABLE IF NOT EXISTS tools (
  code tool_type PRIMARY KEY
);

INSERT INTO tools (code) VALUES ('T1'), ('T2'), ('T3'), ('T4'), ('T5'), ('T6');
