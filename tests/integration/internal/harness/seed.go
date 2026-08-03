package harness

// Bootstrap admin credentials, created through POST /setup on every reset.
//
// Nothing is inserted with SQL any more, and that is the point: the endpoint is
// how a real deployment gets its first user, so exercising it on every reset
// means the tests can never drift from the only bootstrap path that exists. It
// also retires the pre-computed argon2id hash this file used to carry — a
// constant that would have silently rotted the day the hashing parameters
// changed.
const (
	AdminName  = "Integration Admin"
	AdminEmail = "admin@portmaster.local"
	// Satisfies the domain password policy (8+ chars, lower, upper, digit) —
	// the setup endpoint runs the same validation as any other user creation.
	AdminPassword = "Portmaster1"
)
