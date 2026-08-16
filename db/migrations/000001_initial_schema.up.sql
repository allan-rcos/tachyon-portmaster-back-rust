-- Portmaster schema (MariaDB, reached through mysql_async).
--
-- Six tables and one association. Ids are application-generated Snowflakes;
-- only `telemetry_logs` auto-increments, because an append-only log has no
-- identity of its own to generate.
--
-- Every statement is written to survive being applied twice. golang-migrate
-- already tracks what ran in `schema_migrations`, so this is not what protects
-- the normal path — it protects the abnormal one, where someone pipes a
-- migration straight into mariadb to inspect or repair a database mid-incident.
-- Failing there with "table already exists" tells the operator nothing and
-- leaves the rest of the file unapplied. The matching .down.sql is symmetric.
-- See db/README.md.
--
--
-- WHY THE TYPES ARE WHAT THEY ARE
--
-- `BIGINT` and not `BIGINT UNSIGNED`. A Snowflake is 63 bits — timestamp, node,
-- sequence — so it is always positive and never needs the extra bit. The driver
-- decodes an id into `i64`, and an UNSIGNED column would arrive as an unsigned
-- value needing a narrowing cast at the edge: the column type is what makes the
-- read legal.
--
-- `TINYINT` for enums, holding the **ordinal**. The previous schema stored the
-- slug (`'class-2-gases'`), which meant every read parsed a string back into a
-- variant and every rename was a data migration. The ordinal is what the wire
-- already publishes (`common.fbs` declares these enums as `uint8`), so the
-- number in the column is the number the client sees. Signed, again, so it
-- decodes into a Rust integer.
--
-- `BIGINT` for time, holding **epoch milliseconds** — no DATETIME and no
-- TIMESTAMP anywhere. A date type makes the stored instant depend on how it is
-- interpreted: TIMESTAMP converts on write and read against the session time
-- zone and tops out in 2038, and DATETIME stores a wall clock with no zone at
-- all, correct only for as long as every writer agrees on which zone that was.
-- An epoch is one number with one meaning, and a session variable cannot change
-- what it says. It is also the number the wire already publishes, so the value
-- travels from column to client without a conversion in between.
--
-- `DOUBLE` for weights, capacities and densities, matching the `f64` the domain
-- computes with. DECIMAL would be right for money and is wrong here: these are
-- physical measurements, already approximate at the scale, and rounding them at
-- the column would disagree with the arithmetic that produced them.
--
--
-- TIMESTAMPS AND DELETION
--
-- A **strong** entity — one that exists on its own — carries `created_at`,
-- `updated_at` and `deleted_at`, and is removed by soft-delete: the row stays
-- and every read filters `deleted_at IS NULL`.
--
-- Every one of those instants is written by the application, and none has a
-- column default. The domain model already stamps `Utc::now()` when it builds an
-- entity and again on every mutation; a `DEFAULT CURRENT_TIMESTAMP` would throw
-- that value away and record when the INSERT reached the server instead — a
-- different instant, decided by a different clock.
--
-- A **weak** entity — a satellite with no meaning apart from its owner — carries
-- only `created_at` and is removed for real. There is nothing to preserve in a
-- row that only existed while a product sat in a container.
--
-- An **association** (`user_roles`) carries neither. It is not an entity: it has
-- no identity, no lifecycle and nothing to say about when it was written.
-- Replacing a user's roles is a DELETE followed by INSERTs.
--
--
-- UNIQUENESS UNDER SOFT-DELETE
--
-- `users.email` and `containers.code` must be unique **among the living**. A
-- plain UNIQUE would outlive the row and keep an address reserved by someone who
-- was removed years ago — and the application, whose lookups filter deleted rows,
-- would see the address as free and then hit a duplicate-key error it has no
-- reason to expect.
--
-- MariaDB has no partial index, so the constraint is expressed with a generated
-- column: `0` while the row lives, the row's own id once it does not. Living
-- rows therefore collide with each other and with nothing else. The column is
-- never selected or written by the application — every statement it issues names
-- its columns explicitly.

CREATE TABLE IF NOT EXISTS roles (
    id           BIGINT       NOT NULL,
    name         VARCHAR(255) NOT NULL,
    -- MariaDB implements JSON as LONGTEXT plus a validity CHECK, which is why
    -- the driver reads this column as text and parses it itself. Declaring JSON
    -- rather than LONGTEXT buys that check for free.
    permissions  JSON         NOT NULL,
    -- The normalised key the LIKE filters match against, written by the
    -- repository. Searching `name` directly would make the filter depend on
    -- accents and case.
    search_name  VARCHAR(255) NOT NULL,
    created_at   BIGINT       NOT NULL,
    updated_at   BIGINT       NOT NULL,
    deleted_at   BIGINT       NULL,
    PRIMARY KEY (id),
    KEY idx_roles_search_name (search_name),
    -- The shape every listing has: alive, ordered by id. Leading with
    -- `deleted_at` lets the keyset walk the index instead of filtering after it.
    KEY idx_roles_live (deleted_at, id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS users (
    id             BIGINT       NOT NULL,
    name           VARCHAR(255) NOT NULL,
    email          VARCHAR(255) NOT NULL,
    password_hash  VARCHAR(255) NOT NULL,
    created_at     BIGINT       NOT NULL,
    updated_at     BIGINT       NOT NULL,
    deleted_at     BIGINT       NULL,
    live_key       BIGINT       AS (IF(deleted_at IS NULL, 0, id)) STORED,
    PRIMARY KEY (id),
    UNIQUE KEY uq_users_email_live (email, live_key),
    KEY idx_users_live (deleted_at, id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Which roles a user holds. An association, not an entity: no id, no
-- timestamps, and no soft-delete — a grant that was taken away simply is not
-- there. The cascades are a backstop; users and roles are soft-deleted, so
-- nothing normally triggers them.
CREATE TABLE IF NOT EXISTS user_roles (
    user_id  BIGINT NOT NULL,
    role_id  BIGINT NOT NULL,
    PRIMARY KEY (user_id, role_id),
    KEY idx_user_roles_role_id (role_id),
    CONSTRAINT fk_user_roles_user FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
    CONSTRAINT fk_user_roles_role FOREIGN KEY (role_id) REFERENCES roles (id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS products (
    id           BIGINT       NOT NULL,
    name         VARCHAR(255) NOT NULL,
    density      DOUBLE       NOT NULL DEFAULT 0,
    -- Ordinal of Domain RiskClass / API.Fbs.Common.RiskClass; 9 is "unclassified"
    -- and is a class like any other, not a null.
    risk_class   TINYINT      NOT NULL,
    search_name  VARCHAR(255) NOT NULL,
    created_at   BIGINT       NOT NULL,
    updated_at   BIGINT       NOT NULL,
    deleted_at   BIGINT       NULL,
    PRIMARY KEY (id),
    KEY idx_products_search_name (search_name),
    KEY idx_products_live (deleted_at, id),
    CONSTRAINT ck_products_risk_class CHECK (risk_class BETWEEN 0 AND 9)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS containers (
    id              BIGINT       NOT NULL,
    code            VARCHAR(255) NOT NULL,
    -- Denormalised sum of container_items.weight. The domain keeps the two in
    -- step inside one transaction; nothing here recomputes it.
    current_weight  DOUBLE       NOT NULL DEFAULT 0,
    max_capacity    DOUBLE       NOT NULL DEFAULT 0,
    -- Ordinal of Domain ContainerStatus / API.Fbs.Common.ContainerStatus.
    status          TINYINT      NOT NULL,
    search_code     VARCHAR(255) NOT NULL,
    created_at      BIGINT       NOT NULL,
    updated_at      BIGINT       NOT NULL,
    deleted_at      BIGINT       NULL,
    live_key        BIGINT       AS (IF(deleted_at IS NULL, 0, id)) STORED,
    PRIMARY KEY (id),
    UNIQUE KEY uq_containers_code_live (code, live_key),
    KEY idx_containers_search_code (search_code),
    KEY idx_containers_live (deleted_at, id),
    -- The occupancy panel counts by status over the living set, and the listing
    -- filters by it.
    KEY idx_containers_live_status (deleted_at, status),
    CONSTRAINT ck_containers_status CHECK (status BETWEEN 0 AND 3)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- What a container currently holds. Weak: one row per (container, product),
-- meaningless without its container, hard-deleted when the product comes out.
CREATE TABLE IF NOT EXISTS container_items (
    container_id  BIGINT   NOT NULL,
    product_id    BIGINT   NOT NULL,
    quantity      DOUBLE   NOT NULL DEFAULT 0,
    weight        DOUBLE   NOT NULL DEFAULT 0,
    created_at    BIGINT   NOT NULL,
    PRIMARY KEY (container_id, product_id),
    KEY idx_container_items_product (product_id),
    CONSTRAINT fk_container_items_container FOREIGN KEY (container_id) REFERENCES containers (id) ON DELETE CASCADE,
    CONSTRAINT fk_container_items_product FOREIGN KEY (product_id) REFERENCES products (id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- What happened to a container. Append-only history: nothing updates or deletes
-- a row here, so there is no `updated_at` to keep and no soft-delete to apply.
--
-- `id` auto-increments because the entry has no identity to generate — it is the
-- order it was written in, and that order is what the "recent logs" window reads.
CREATE TABLE IF NOT EXISTS telemetry_logs (
    id            BIGINT   NOT NULL AUTO_INCREMENT,
    container_id  BIGINT   NOT NULL,
    -- Ordinal of Domain TelemetryEvent / API.Fbs.Common.TelemetryEvent.
    event         TINYINT  NOT NULL,
    description   TEXT     NULL,
    -- When the event happened, written explicitly by the repository. Not a
    -- `created_at`: the row and the event are the same thing here.
    timestamp     BIGINT   NOT NULL,
    PRIMARY KEY (id),
    -- Descending id inside a container is exactly how the recent window is read.
    KEY idx_telemetry_container (container_id, id),
    CONSTRAINT fk_telemetry_container FOREIGN KEY (container_id) REFERENCES containers (id) ON DELETE CASCADE,
    CONSTRAINT ck_telemetry_event CHECK (event BETWEEN 0 AND 1)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
