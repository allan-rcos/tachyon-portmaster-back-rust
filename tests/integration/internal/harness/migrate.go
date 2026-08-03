package harness

import (
	"errors"
	"fmt"

	"github.com/golang-migrate/migrate/v4"
	_ "github.com/golang-migrate/migrate/v4/database/mysql"
	_ "github.com/golang-migrate/migrate/v4/source/file"
)

// migrateURL is the golang-migrate DSN for a database on the shared MariaDB,
// reached through the host-mapped port. multiStatements lets each migration
// file carry several DDL statements.
func migrateURL(mappedPort, dbName string) string {
	return fmt.Sprintf("mysql://root:%s@tcp(127.0.0.1:%s)/%s?multiStatements=true", dbRootPass, mappedPort, dbName)
}

func migrateUp(sourceURL, dsn string) error {
	m, err := migrate.New(sourceURL, dsn)
	if err != nil {
		return fmt.Errorf("migrate open: %w", err)
	}
	defer m.Close()

	if err := m.Up(); err != nil && !errors.Is(err, migrate.ErrNoChange) {
		return fmt.Errorf("migrate up: %w", err)
	}
	return nil
}

// migrateDrop removes every table (including schema_migrations) but leaves the
// database itself, so the API's warm connection pool stays valid across a reset.
func migrateDrop(sourceURL, dsn string) error {
	m, err := migrate.New(sourceURL, dsn)
	if err != nil {
		return fmt.Errorf("migrate open: %w", err)
	}
	defer m.Close()

	if err := m.Drop(); err != nil {
		return fmt.Errorf("migrate drop: %w", err)
	}
	return nil
}
