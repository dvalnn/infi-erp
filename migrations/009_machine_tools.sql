CREATE TABLE IF NOT EXISTS machine_tools (
  machine char(2) NOT NULL REFERENCES machines(code),
  tool char(2) NOT NULL REFERENCES tools(code),
  PRIMARY KEY (machine, tool)
);

INSERT INTO machine_tools (machine, tool)
VALUES ('M1', 'T1'),
       ('M1', 'T2'),
       ('M1', 'T3'),

       ('M2', 'T1'),
       ('M2', 'T2'),
       ('M2', 'T3'),

       ('M3', 'T1'),
       ('M3', 'T4'),
       ('M3', 'T5'),

       ('M4', 'T1'),
       ('M4', 'T4'),
       ('M4', 'T6');
