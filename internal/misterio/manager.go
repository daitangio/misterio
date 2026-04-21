package misterio

import (
	"errors"
	"fmt"
	"io"
	"os"
	"os/exec"
	"path/filepath"
	"sort"
	"strings"
	"time"

	"github.com/daitangio/misterio/internal/config"
)

const Version = "0.1.6-dev"

type RunOptions struct {
	Hosts         []string
	ListOnly      bool
	SingleRole    string
	DockerCommand []string
}

type Manager struct {
	Home   string
	Stdout io.Writer
	Now    func() time.Time
}

func NewManager(home string, stdout io.Writer) *Manager {
	if stdout == nil {
		stdout = os.Stdout
	}
	return &Manager{
		Home:   home,
		Stdout: stdout,
		Now:    time.Now,
	}
}

func (m *Manager) Run(opts RunOptions) error {
	if err := m.verifyHome(); err != nil {
		return err
	}

	hosts, err := m.resolveHosts(opts.Hosts)
	if err != nil {
		return err
	}

	fmt.Fprintf(m.Stdout, "HOSTS:%v MISTERIO HOME:%s\n", hosts, m.Home)
	if opts.ListOnly {
		return m.listRoles(hosts)
	}

	configFile := filepath.Join(m.Home, "hosts", "misterio.toml")
	hostConfig, configExists, err := m.loadConfig(configFile)
	if err != nil {
		return err
	}
	if !configExists {
		fmt.Fprintf(m.Stdout, "%s not defined (see documentation for happy features)\n", configFile)
	}

	for _, host := range hosts {
		if strings.Contains(host, "localhost") {
			return fmt.Errorf("use misterio.toml not localhost for special needs")
		}

		runtimeCfg := runtimeConfig{DockerHost: fmt.Sprintf("ssh://%s", host)}
		if configExists {
			if cfg, ok := hostConfig.Hosts[host]; ok {
				if cfg.Docker.Context != "" || cfg.Docker.Host != "" {
					fmt.Fprintf(
						m.Stdout,
						"Config for %s: context=%q host=%q\n",
						host,
						cfg.Docker.Context,
						cfg.Docker.Host,
					)
				}
				runtimeCfg.DockerContext = cfg.Docker.Context
				if cfg.Docker.Host != "" {
					runtimeCfg.DockerHost = cfg.Docker.Host
				} else if cfg.Docker.Context != "" {
					runtimeCfg.DockerHost = ""
				}
			}
		}

		if err := m.processHost(host, runtimeCfg, opts.SingleRole, opts.DockerCommand); err != nil {
			return err
		}
	}

	return nil
}

func (m *Manager) AddRoles(targetHost string, roles []string, build bool) error {
	basePort := 7000
	for _, role := range roles {
		targetDir := filepath.Join(m.Home, "hosts", targetHost)
		if err := os.MkdirAll(targetDir, 0o755); err != nil {
			return err
		}

		envPath := filepath.Join(targetDir, role+".env")
		if _, err := os.Stat(envPath); err == nil {
			return fmt.Errorf("role %s already exists as %s", role, envPath)
		} else if !os.IsNotExist(err) {
			return err
		}

		if err := m.writeRoleEnv(envPath, role, basePort); err != nil {
			return err
		}

		basePort = determineFixedPort(role, basePort) + 1
		if build {
			if err := m.Run(RunOptions{
				Hosts:         []string{targetHost},
				SingleRole:    role,
				DockerCommand: []string{"build"},
			}); err != nil {
				return err
			}
		}
	}

	return nil
}

func (m *Manager) MoveRole(role, sourceHost, destinationHost string) error {
	if err := m.Run(RunOptions{
		Hosts:         []string{sourceHost},
		SingleRole:    role,
		DockerCommand: []string{"down"},
	}); err != nil {
		return err
	}

	src := filepath.Join(m.Home, "hosts", sourceHost, role+".env")
	dst := filepath.Join(m.Home, "hosts", destinationHost, role+".env")
	if err := os.MkdirAll(filepath.Dir(dst), 0o755); err != nil {
		return err
	}
	fmt.Fprintf(m.Stdout, "%s -> %s\n", src, dst)
	if err := os.Rename(src, dst); err != nil {
		return err
	}

	return m.Run(RunOptions{
		Hosts:         []string{destinationHost},
		SingleRole:    role,
		DockerCommand: []string{"up", "-d"},
	})
}

func (m *Manager) RemoveRoles(sourceHost string, roles []string) error {
	for _, role := range roles {
		if err := m.Run(RunOptions{
			Hosts:         []string{sourceHost},
			SingleRole:    role,
			DockerCommand: []string{"down"},
		}); err != nil {
			return err
		}

		src := filepath.Join(m.Home, "hosts", sourceHost, role+".env")
		fmt.Fprintf(m.Stdout, "Moving %s to the attic\n", src)
		atticDir := filepath.Join(m.Home, "attic", sourceHost)
		if err := os.MkdirAll(atticDir, 0o755); err != nil {
			return err
		}
		dst := filepath.Join(atticDir, role+".env")
		if err := os.Rename(src, dst); err != nil {
			return err
		}

		if err := m.Run(RunOptions{
			Hosts:         []string{sourceHost},
			DockerCommand: []string{"@rebuild"},
		}); err != nil {
			return err
		}
	}
	return nil
}

type runtimeConfig struct {
	DockerContext string
	DockerHost    string
}

func (m *Manager) verifyHome() error {
	var missing []string
	for _, dir := range []string{"hosts", "roles"} {
		path := filepath.Join(m.Home, dir)
		info, err := os.Stat(path)
		if err != nil || !info.IsDir() {
			missing = append(missing, path)
		}
	}
	if len(missing) > 0 {
		return fmt.Errorf("home dir validation errors: missing %s", strings.Join(missing, ", "))
	}
	return nil
}

func (m *Manager) resolveHosts(requested []string) ([]string, error) {
	if len(requested) > 0 {
		return requested, nil
	}

	entries, err := os.ReadDir(filepath.Join(m.Home, "hosts"))
	if err != nil {
		return nil, err
	}

	var hosts []string
	for _, entry := range entries {
		if entry.IsDir() {
			hosts = append(hosts, entry.Name())
		}
	}
	sort.Strings(hosts)
	return hosts, nil
}

func (m *Manager) listRoles(hosts []string) error {
	for _, host := range hosts {
		fmt.Fprintf(m.Stdout, "Roles for %s\n", host)
		entries, err := os.ReadDir(filepath.Join(m.Home, "hosts", host))
		if err != nil {
			fmt.Fprintf(m.Stdout, "No roles for %s\n", host)
			continue
		}
		var names []string
		for _, entry := range entries {
			names = append(names, entry.Name())
		}
		sort.Strings(names)
		for _, name := range names {
			fmt.Fprintf(m.Stdout, "\t%s\n", name)
		}
	}
	return nil
}

func (m *Manager) loadConfig(path string) (config.File, bool, error) {
	cfg, err := config.ParseFile(path)
	if err == nil {
		return cfg, true, nil
	}
	if os.IsNotExist(err) {
		return config.File{}, false, nil
	}
	return config.File{}, false, err
}

func (m *Manager) processHost(host string, cfg runtimeConfig, singleRole string, dockerCommand []string) error {
	hostDir := filepath.Join(m.Home, "hosts", host)
	entries, err := os.ReadDir(hostDir)
	if err != nil {
		return err
	}

	var names []string
	for _, entry := range entries {
		names = append(names, entry.Name())
	}
	sort.Strings(names)

	for _, name := range names {
		if !strings.HasSuffix(name, ".env") {
			fmt.Fprintf(m.Stdout, "WARN Ignored not-env file: %s\n", name)
			continue
		}
		if singleRole != "" && !strings.Contains(name, singleRole) {
			continue
		}
		if err := m.processRole(filepath.Join(hostDir, name), dockerCommand, cfg); err != nil {
			return err
		}
	}
	return nil
}

func (m *Manager) processRole(envPath string, dockerCommand []string, cfg runtimeConfig) error {
	if len(dockerCommand) == 1 && strings.HasPrefix(dockerCommand[0], "@") {
		switch dockerCommand[0] {
		case "@rebuild":
			if err := m.runCompose(envPath, []string{"down"}, cfg); err != nil {
				return err
			}
			return m.runCompose(envPath, []string{"up", "--build", "-d"}, cfg)
		case "@upgrade":
			if err := m.runCompose(envPath, []string{"pull"}, cfg); err != nil {
				return err
			}
			if err := m.runCompose(envPath, []string{"down"}, cfg); err != nil {
				return err
			}
			return m.runCompose(envPath, []string{"up", "--build", "-d"}, cfg)
		default:
			return fmt.Errorf("unknown alias: %s", dockerCommand[0])
		}
	}

	return m.runCompose(envPath, dockerCommand, cfg)
}

func (m *Manager) runCompose(envPath string, dockerCommand []string, cfg runtimeConfig) error {
	roleName := roleNameFromEnvFile(filepath.Base(envPath))
	roleDir := filepath.Join(m.Home, "roles", roleName)
	fullCommand := append([]string{"compose"}, dockerCommand...)

	fmt.Fprintf(m.Stdout, "==== %s %s \t-> %v\n", cfg.DockerHost, roleName, append([]string{"docker"}, fullCommand...))
	if err := copyFile(envPath, filepath.Join(roleDir, ".env")); err != nil {
		return err
	}

	cmd := exec.Command("docker", fullCommand...)
	cmd.Dir = roleDir
	cmd.Stdout = m.Stdout
	cmd.Stderr = m.Stdout
	cmd.Env = updatedEnv(os.Environ(), cfg)

	if err := cmd.Run(); err != nil {
		var exitErr *exec.ExitError
		if strings.Contains(err.Error(), "executable file not found") {
			return err
		}
		if errors.As(err, &exitErr) {
			fmt.Fprintf(m.Stdout, "%s::%s Failed with return code %d\n", cfg.DockerHost, roleName, exitErr.ExitCode())
			return nil
		}
		return err
	}

	return nil
}
