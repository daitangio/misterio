package main

import (
	"os"
	"testing"
)

func TestParseArgsWithFlagSet(t *testing.T) {
	wd, err := os.Getwd()
	if err != nil {
		t.Fatalf("Getwd() error = %v", err)
	}

	t.Setenv("MISTERIO_HOME", "")
	t.Setenv("MISTERIO_SINGLE_ROLE", "")

	opts, err := parseArgs([]string{"--home", "/tmp/demo", "-h", "host-a", "--misterio-host=host-b", "--list", "-r", "pgvector", "ps"})
	if err != nil {
		t.Fatalf("parseArgs() error = %v", err)
	}

	if opts.home != "/tmp/demo" {
		t.Fatalf("home = %q, want %q", opts.home, "/tmp/demo")
	}
	if len(opts.hosts) != 2 || opts.hosts[0] != "host-a" || opts.hosts[1] != "host-b" {
		t.Fatalf("hosts = %v, want [host-a host-b]", opts.hosts)
	}
	if !opts.listOnly {
		t.Fatalf("listOnly = false, want true")
	}
	if opts.singleRole != "pgvector" {
		t.Fatalf("singleRole = %q, want %q", opts.singleRole, "pgvector")
	}
	if len(opts.command) != 1 || opts.command[0] != "ps" {
		t.Fatalf("command = %v, want [ps]", opts.command)
	}

	opts, err = parseArgs([]string{"@rebuild"})
	if err != nil {
		t.Fatalf("parseArgs() error = %v", err)
	}
	if opts.home != wd {
		t.Fatalf("home = %q, want %q", opts.home, wd)
	}
	if len(opts.command) != 1 || opts.command[0] != "@rebuild" {
		t.Fatalf("command = %v, want [@rebuild]", opts.command)
	}
}

func TestParseArgsNoListOverridesList(t *testing.T) {
	t.Setenv("MISTERIO_HOME", "")
	t.Setenv("MISTERIO_SINGLE_ROLE", "")

	opts, err := parseArgs([]string{"--list", "--no-list", "ps"})
	if err != nil {
		t.Fatalf("parseArgs() error = %v", err)
	}
	if opts.listOnly {
		t.Fatalf("listOnly = true, want false")
	}
}
