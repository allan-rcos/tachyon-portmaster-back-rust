-- Reverses 000001_initial_schema.up.sql, dropping in reverse
-- foreign-key-dependency order.
--
-- The test harness resets by dropping and re-migrating, so this file runs as
-- often as the `up` does — an incorrect order here breaks every integration
-- test rather than just the migration.

DROP TABLE IF EXISTS telemetry_logs;
DROP TABLE IF EXISTS container_items;
DROP TABLE IF EXISTS containers;
DROP TABLE IF EXISTS products;
DROP TABLE IF EXISTS user_roles;
DROP TABLE IF EXISTS users;
DROP TABLE IF EXISTS roles;
