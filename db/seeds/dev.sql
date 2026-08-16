-- Development seed data — applied ONLY by the dev docker-compose stack, after
-- migrations. Never loaded by the integration tests or CI; those build their own
-- data through factories and POST /setup.
--
-- **No user and no role here.** Bootstrapping is `POST /setup`, which creates the
-- first user together with a role holding every registered permission:
--
--   curl -X POST localhost:8000/setup -H 'Content-Type: application/json' \
--        -d '{"name":"Admin","email":"admin@portmaster.local","password":"Portmaster1"}'
--
-- That is the path a real deployment uses, so the dev stack uses it too. Seeding
-- a user in SQL instead would mean carrying a pre-computed argon2id hash and a
-- hand-copied list of permission slugs in this file — and that list had already
-- drifted three slugs behind the code before anyone noticed, precisely because
-- nothing exercised it.
--
-- Ids are fixed small Snowflakes for readability; the application Base62-encodes
-- them at the edge, so container 1 is `/containers/1`.
--
-- Enum columns hold **ordinals**, matching the domain enums and common.fbs:
--
--   risk_class: 1 = Class2Gases, 7 = Class8CorrosiveSubstances
--   status:     0 = Empty, 1 = Loading
--
-- The CHECK constraints on those columns reject a typo here at seed time rather
-- than letting it surface as a failed read months later.
--
-- Timestamps are epoch milliseconds, and the seed writes them the same way the
-- application does — the columns have no default to fall back on. The value is
-- the fixed literal 1735689600000 (2025-01-01T00:00:00Z) rather than a call to
-- the server's clock: a seed that stamped "now" would make every re-run produce
-- a different database, and these rows are meant to be the same ones every time.
--
-- **Idempotent.** The compose `seed` service runs on every `docker compose up`,
-- while the `db_data` volume survives everything short of `down -v` — so the
-- second start always finds these rows already there. As plain INSERTs this file
-- failed on the duplicate primary key, and because `app` waits on
-- `seed: service_completed_successfully`, a failed seed meant the API never
-- started at all. Re-seeding a running deployment is the normal case, not an
-- error case.
--
-- ON DUPLICATE KEY UPDATE rather than INSERT IGNORE: re-seeding should bring a
-- row back to the value declared below, not silently accept whatever a developer
-- left in the table. Both survive a re-run; only this one converges.

INSERT INTO products (id, name, density, risk_class, search_name, created_at, updated_at) VALUES
    (1, 'Liquid Nitrogen',  0.807, 1, 'liquid nitrogen',  1735689600000, 1735689600000),
    (2, 'Sodium Hydroxide', 2.13,  7, 'sodium hydroxide', 1735689600000, 1735689600000)
ON DUPLICATE KEY UPDATE
    name        = VALUES(name),
    density     = VALUES(density),
    risk_class  = VALUES(risk_class),
    search_name = VALUES(search_name),
    created_at  = VALUES(created_at),
    updated_at  = VALUES(updated_at);

-- The cargo of the two seeded containers is rewritten together with them.
-- `current_weight` is a denormalised sum of `container_items.weight`, and the
-- application keeps the two in step inside a transaction; a seed that wrote one
-- without the other would create the one inconsistency no endpoint repairs.
--
-- Scoped to ids 1 and 2: containers a developer created through the API are not
-- this file's business. `telemetry_logs` is deliberately left alone — it is an
-- append-only history, and an old entry does not contradict the row below.
DELETE FROM container_items WHERE container_id IN (1, 2);

INSERT INTO containers (id, code, current_weight, max_capacity, status, search_code, created_at, updated_at) VALUES
    (1, 'CT-0001',   0, 1000, 0, 'ct-0001', 1735689600000, 1735689600000),
    -- 100 units of product 2, at density 2.13 → 213 kg. Written out rather than
    -- computed so that reading this file tells you the state it produces.
    (2, 'CT-0002', 213, 1000, 1, 'ct-0002', 1735689600000, 1735689600000)
ON DUPLICATE KEY UPDATE
    code           = VALUES(code),
    current_weight = VALUES(current_weight),
    max_capacity   = VALUES(max_capacity),
    status         = VALUES(status),
    search_code    = VALUES(search_code),
    created_at     = VALUES(created_at),
    updated_at     = VALUES(updated_at);

INSERT INTO container_items (container_id, product_id, quantity, weight, created_at) VALUES
    (2, 2, 100, 213, 1735689600000)
ON DUPLICATE KEY UPDATE
    quantity   = VALUES(quantity),
    weight     = VALUES(weight),
    created_at = VALUES(created_at);
