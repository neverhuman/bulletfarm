-- 002: kernel defects 1-3 (docs/plan/close-all-gaps.md §1).
-- Applied inside one transaction with foreign_keys=OFF (SQLite ALTER TABLE procedure);
-- storage.rs runs PRAGMA foreign_key_check before committing.

-- Defect 3: principal kinds beyond human|agent (spec §13).
CREATE TABLE principals_v2(
  id TEXT PRIMARY KEY,
  kind TEXT NOT NULL CHECK(kind IN ('human','agent','runner','verifier','publisher')),
  active INTEGER NOT NULL CHECK(active IN (0,1))
);
INSERT INTO principals_v2(id,kind,active) SELECT id,kind,active FROM principals;
DROP TABLE principals;
ALTER TABLE principals_v2 RENAME TO principals;

-- Defect 2: one event per accepted command, written in the same transaction as the
-- command and operation rows. kind is the command kind; payload_json records the
-- actor kind and the operation outcome.
CREATE TABLE events(
  seq INTEGER PRIMARY KEY AUTOINCREMENT,
  at TEXT NOT NULL,
  actor_id TEXT NOT NULL REFERENCES principals(id),
  kind TEXT NOT NULL,
  command_id TEXT NOT NULL,
  operation_id TEXT NOT NULL,
  payload_json TEXT NOT NULL
);

-- Defect 1: command ids are namespaced per actor (spec §20.2-c, BF3-003-AC02).
CREATE TABLE commands_v2(
  actor_id TEXT NOT NULL REFERENCES principals(id),
  command_id TEXT NOT NULL,
  actor_kind TEXT NOT NULL,
  kind TEXT NOT NULL,
  body_sha TEXT NOT NULL,
  raw_json TEXT NOT NULL,
  created_at TEXT NOT NULL,
  PRIMARY KEY(actor_id, command_id)
);
INSERT INTO commands_v2(actor_id,command_id,actor_kind,kind,body_sha,raw_json,created_at)
  SELECT c.actor_id,c.command_id,p.kind,c.kind,c.body_sha,c.raw_json,c.created_at
  FROM commands c JOIN principals p ON p.id=c.actor_id;
DROP TABLE commands;
ALTER TABLE commands_v2 RENAME TO commands;

CREATE TABLE operations_v2(
  id TEXT PRIMARY KEY,
  actor_id TEXT NOT NULL REFERENCES principals(id),
  command_id TEXT NOT NULL,
  actor_kind TEXT NOT NULL,
  status TEXT NOT NULL,
  result_json TEXT NOT NULL,
  UNIQUE(actor_id, command_id),
  FOREIGN KEY(actor_id, command_id) REFERENCES commands(actor_id, command_id)
);
INSERT INTO operations_v2(id,actor_id,command_id,actor_kind,status,result_json)
  SELECT o.id,o.actor_id,o.command_id,p.kind,o.status,o.result_json
  FROM operations o JOIN principals p ON p.id=o.actor_id;
DROP TABLE operations;
ALTER TABLE operations_v2 RENAME TO operations;

INSERT INTO schema_migrations VALUES(2);
