package main

import (
	"flag"
	"fmt"
	"os"

	"github.com/daitangio/misterio/internal/misterio"
)

type options struct {
	home       string
	hosts      []string
	listOnly   bool
	singleRole string
	command    []string
}

type stringSliceFlag struct {
	values []string
}

func (s *stringSliceFlag) String() string {
	return fmt.Sprint(s.values)
}

func (s *stringSliceFlag) Set(value string) error {
	s.values = append(s.values, value)
	return nil
}

type boolSetter struct {
	target *bool
	value  bool
}

func (b boolSetter) String() string {
	if b.target == nil {
		return "false"
	}
	return fmt.Sprintf("%t", *b.target)
}

func (b boolSetter) Set(string) error {
	*b.target = b.value
	return nil
}

func (b boolSetter) IsBoolFlag() bool {
	return true
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

	fs := flag.NewFlagSet("misterio", flag.ContinueOnError)
	fs.SetOutput(os.Stderr)
	fs.Usage = func() {
		printUsage(os.Stderr)
	}

	var hosts stringSliceFlag
	var showHelp bool
	var showVersion bool

	fs.StringVar(&opts.home, "home", opts.home, "")
	fs.Var(&hosts, "h", "")
	fs.Var(&hosts, "misterio-host", "")
	fs.StringVar(&opts.singleRole, "r", opts.singleRole, "")
	fs.StringVar(&opts.singleRole, "single-role", opts.singleRole, "")
	fs.BoolVar(&opts.listOnly, "list", false, "")
	fs.Var(boolSetter{target: &opts.listOnly, value: false}, "no-list", "")
	fs.BoolVar(&showHelp, "help", false, "")
	fs.BoolVar(&showVersion, "version", false, "")

	if err := fs.Parse(args); err != nil {
		return options{}, err
	}

	if showHelp {
		printUsage(os.Stdout)
		os.Exit(0)
	}

	if showVersion {
		fmt.Println(misterio.Version)
		os.Exit(0)
	}

	opts.hosts = hosts.values
	opts.command = fs.Args()
	return opts, nil
}

func printUsage(w *os.File) {
	fmt.Fprintln(w, "Usage: misterio [--home PATH] [-h HOST ...] [--list] [-r ROLE] [--] [docker compose args]")
	fmt.Fprintln(w, "Examples:")
	fmt.Fprintln(w, "  misterio --home ./misterio_project @rebuild")
	fmt.Fprintln(w, "  misterio -h wonderboy --single-role elastic-service ps")
}
