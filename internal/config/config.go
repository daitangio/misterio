package config

import (
	"bufio"
	"bytes"
	"fmt"
	"os"
	"strconv"
	"strings"
)

type DockerConfig struct {
	Context string
	Host    string
}

type HostConfig struct {
	Docker DockerConfig
}

type File struct {
	Hosts map[string]HostConfig
}

func Parse(data []byte) (File, error) {
	result := File{Hosts: make(map[string]HostConfig)}
	scanner := bufio.NewScanner(bytes.NewReader(data))
	var section []string

	for lineNo := 1; scanner.Scan(); lineNo++ {
		line := strings.TrimSpace(stripComment(scanner.Text()))
		if line == "" {
			continue
		}

		if strings.HasPrefix(line, "[") && strings.HasSuffix(line, "]") {
			name := strings.TrimSpace(line[1 : len(line)-1])
			if name == "" {
				return File{}, fmt.Errorf("line %d: empty section name", lineNo)
			}
			section = strings.Split(name, ".")
			continue
		}

		parts := strings.SplitN(line, "=", 2)
		if len(parts) != 2 {
			return File{}, fmt.Errorf("line %d: invalid assignment", lineNo)
		}
		key := strings.TrimSpace(parts[0])
		value, err := parseValue(parts[1])
		if err != nil {
			return File{}, fmt.Errorf("line %d: %w", lineNo, err)
		}

		if len(section) == 2 && section[1] == "docker" {
			hostCfg := result.Hosts[section[0]]
			switch key {
			case "context":
				hostCfg.Docker.Context = value
			case "host":
				hostCfg.Docker.Host = value
			}
			result.Hosts[section[0]] = hostCfg
		}
	}

	if err := scanner.Err(); err != nil {
		return File{}, err
	}
	return result, nil
}

func ParseFile(path string) (File, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return File{}, err
	}
	return Parse(data)
}

func parseValue(raw string) (string, error) {
	value := strings.TrimSpace(raw)
	if value == "" {
		return "", nil
	}
	if strings.HasPrefix(value, "\"") {
		parsed, err := strconv.Unquote(value)
		if err != nil {
			return "", fmt.Errorf("invalid quoted value: %w", err)
		}
		return parsed, nil
	}
	return value, nil
}

func stripComment(line string) string {
	inQuotes := false
	for i, r := range line {
		switch r {
		case '"':
			inQuotes = !inQuotes
		case '#':
			if !inQuotes {
				return line[:i]
			}
		}
	}
	return line
}
