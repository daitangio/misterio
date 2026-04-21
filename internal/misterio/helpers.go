package misterio

import (
	"fmt"
	"io"
	"os"
	"path/filepath"
	"strconv"
	"strings"
)

func roleNameFromEnvFile(envFile string) string {
	if strings.Contains(envFile, "@") {
		return strings.SplitN(envFile, "@", 2)[0]
	}
	return strings.TrimSuffix(envFile, ".env")
}

func determineInstanceName(role string) string {
	if !strings.Contains(role, "@") {
		return strings.ToLower(role)
	}

	parts := strings.SplitN(role, "@", 2)
	return strings.ToLower(parts[0] + "_" + parts[1])
}

func determineFixedPort(role string, basePort int) int {
	idx := 0
	if strings.Contains(role, "@") {
		parts := strings.SplitN(role, "@", 2)
		value, err := strconv.ParseInt(parts[1], 36, 64)
		if err == nil {
			decimal := strconv.FormatInt(value, 10)
			if len(decimal) > 2 {
				decimal = decimal[:2]
			}
			idx, _ = strconv.Atoi(decimal)
		}
	}
	return basePort + len(role) + idx
}

func (m *Manager) writeRoleEnv(path string, role string, basePort int) error {
	file, err := os.Create(path)
	if err != nil {
		return err
	}
	defer file.Close()

	instanceName := determineInstanceName(role)
	if err := writeProp(file, "MISTERIO_CREATION_USER", os.Getenv("USER")); err != nil {
		return err
	}
	if err := writeProp(file, "MISTERIO_CREATION_DATE", m.Now().Format("2006-01-02 15:04:05")); err != nil {
		return err
	}
	if err := writeProp(file, strings.ToUpper(instanceName)+"_HOME", "/opt/"+instanceName); err != nil {
		return err
	}
	if err := writeProp(file, "MISTERIO_MAGIPORT", determineFixedPort(role, basePort)); err != nil {
		return err
	}
	return nil
}

func writeProp(w io.Writer, key string, value any) error {
	raw := fmt.Sprint(value)
	if raw == "" {
		raw = "unknown"
	}

	var prop string
	switch value.(type) {
	case string:
		if strings.Contains(raw, " ") {
			prop = fmt.Sprintf("%s=%q", strings.ToUpper(key), raw)
		} else {
			prop = fmt.Sprintf("%s=%s", strings.ToUpper(key), raw)
		}
	default:
		prop = fmt.Sprintf("%s=%s", strings.ToUpper(key), raw)
	}

	if _, err := io.WriteString(w, prop+"\n"); err != nil {
		return err
	}
	return nil
}

func copyFile(src string, dst string) error {
	source, err := os.Open(src)
	if err != nil {
		return err
	}
	defer source.Close()

	if err := os.MkdirAll(filepath.Dir(dst), 0o755); err != nil {
		return err
	}

	target, err := os.Create(dst)
	if err != nil {
		return err
	}
	defer target.Close()

	if _, err := io.Copy(target, source); err != nil {
		return err
	}
	return target.Close()
}

func updatedEnv(base []string, cfg runtimeConfig) []string {
	env := map[string]string{}
	for _, entry := range base {
		parts := strings.SplitN(entry, "=", 2)
		if len(parts) == 2 {
			env[parts[0]] = parts[1]
		}
	}

	if cfg.DockerHost == "" {
		delete(env, "DOCKER_HOST")
	} else {
		env["DOCKER_HOST"] = cfg.DockerHost
	}

	if cfg.DockerContext == "" {
		delete(env, "DOCKER_CONTEXT")
	} else {
		env["DOCKER_CONTEXT"] = cfg.DockerContext
	}

	out := make([]string, 0, len(env))
	for key, value := range env {
		out = append(out, key+"="+value)
	}
	return out
}
