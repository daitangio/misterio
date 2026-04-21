package misterio

import "testing"

func TestDetermineInstanceName(t *testing.T) {
	t.Parallel()

	if got := determineInstanceName("pgvector@1"); got != "pgvector_1" {
		t.Fatalf("determineInstanceName() = %q, want %q", got, "pgvector_1")
	}

	if got := determineInstanceName("watchtower"); got != "watchtower" {
		t.Fatalf("determineInstanceName() = %q, want %q", got, "watchtower")
	}
}

func TestDetermineFixedPort(t *testing.T) {
	t.Parallel()

	if got := determineFixedPort("pgvector", 7000); got != 7008 {
		t.Fatalf("determineFixedPort() = %d, want %d", got, 7008)
	}

	if got := determineFixedPort("pgvector@1", 7000); got != 7011 {
		t.Fatalf("determineFixedPort() = %d, want %d", got, 7011)
	}
}
