package config

import "testing"

func TestParseDockerHostConfig(t *testing.T) {
	t.Parallel()

	data := []byte(`
# comment
[_system]
misterio_api=1.6

[orbstack.docker]
context="orbstack"
host=""
`)

	cfg, err := Parse(data)
	if err != nil {
		t.Fatalf("Parse() error = %v", err)
	}

	hostCfg, ok := cfg.Hosts["orbstack"]
	if !ok {
		t.Fatalf("expected host config for orbstack")
	}

	if hostCfg.Docker.Context != "orbstack" {
		t.Fatalf("context = %q, want %q", hostCfg.Docker.Context, "orbstack")
	}

	if hostCfg.Docker.Host != "" {
		t.Fatalf("host = %q, want empty string", hostCfg.Docker.Host)
	}
}
