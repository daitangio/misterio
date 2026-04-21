package main

import (
	"fmt"
	"os"
	"strings"

	"github.com/daitangio/misterio/internal/misterio"
)

type options struct {
	home       string
	hosts      []string
	listOnly   bool
	singleRole string
	command    []string
}

func main() {
	opts, err := parseArgs(os.Args[1:])
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		printUsage(os.Stderr)
		os.Exit(2)
	}

	manager := misterio.NewManager(opts.home, os.Stdout)
	if err := manager.Run(misterio.RunOptions{
		Hosts:         opts.hosts,
		ListOnly:      opts.listOnly,
		SingleRole:    opts.singleRole,
		DockerCommand: opts.command,
	}); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}

func parseArgs(args []string) (options, error) {
	home, err := os.Getwd()
	if err != nil {
		return options{}, err
	}
	if envHome := os.Getenv("MISTERIO_HOME"); envHome != "" {
		home = envHome
	}

	opts := options{
		home:       home,
		singleRole: os.Getenv("MISTERIO_SINGLE_ROLE"),
	}

	for i := 0; i < len(args); i++ {
		arg := args[i]
		switch {
		case arg == "--":
			opts.command = append(opts.command, args[i+1:]...)
			return opts, nil
		case arg == "--help":
			printUsage(os.Stdout)
			os.Exit(0)
		case arg == "--version":
			fmt.Println(misterio.Version)
			os.Exit(0)
		case arg == "--list":
			opts.listOnly = true
		case arg == "--no-list":
			opts.listOnly = false
		case arg == "--home":
			i++
			if i >= len(args) {
				return options{}, fmt.Errorf("missing value for --home")
			}
			opts.home = args[i]
		case strings.HasPrefix(arg, "--home="):
			opts.home = strings.TrimPrefix(arg, "--home=")
		case arg == "--misterio-host" || arg == "-h":
			i++
			if i >= len(args) {
				return options{}, fmt.Errorf("missing value for %s", arg)
			}
			opts.hosts = append(opts.hosts, args[i])
		case strings.HasPrefix(arg, "--misterio-host="):
			opts.hosts = append(opts.hosts, strings.TrimPrefix(arg, "--misterio-host="))
		case arg == "--single-role" || arg == "-r":
			i++
			if i >= len(args) {
				return options{}, fmt.Errorf("missing value for %s", arg)
			}
			opts.singleRole = args[i]
		case strings.HasPrefix(arg, "--single-role="):
			opts.singleRole = strings.TrimPrefix(arg, "--single-role=")
		default:
			opts.command = append(opts.command, args[i:]...)
			return opts, nil
		}
	}

	return opts, nil
}

func printUsage(w *os.File) {
	fmt.Fprintln(w, "Usage: misterio [--home PATH] [-h HOST ...] [--list] [-r ROLE] [--] [docker compose args]")
	fmt.Fprintln(w, "Examples:")
	fmt.Fprintln(w, "  misterio --home ./misterio_project @rebuild")
	fmt.Fprintln(w, "  misterio -h wonderboy --single-role elastic-service ps")
}
