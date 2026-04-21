package main

import (
	"flag"
	"fmt"
	"os"

	"github.com/daitangio/misterio/internal/misterio"
)

func main() {
	home, err := defaultHome()
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}

	build := false
	showHelp := false

	fs := flag.NewFlagSet("misterio-add", flag.ContinueOnError)
	fs.SetOutput(os.Stderr)
	fs.Usage = printUsage
	fs.StringVar(&home, "home", home, "")
	fs.BoolVar(&build, "build", false, "")
	fs.Var(boolSetter{target: &build, value: false}, "no-build", "")
	fs.BoolVar(&showHelp, "help", false, "")

	if err := fs.Parse(os.Args[1:]); err != nil {
		exitf("%v", err)
	}
	if showHelp {
		printUsage()
		return
	}

	positional := fs.Args()
	if len(positional) < 2 {
		printUsage()
		os.Exit(2)
	}

	manager := misterio.NewManager(home, os.Stdout)
	if err := manager.AddRoles(positional[0], positional[1:], build); err != nil {
		exitf("%v", err)
	}
}

func printUsage() {
	fmt.Println("Usage: misterio-add [--home PATH] [--build|--no-build] TARGET_HOST ROLE [ROLE...]")
}

func exitf(format string, args ...any) {
	fmt.Fprintf(os.Stderr, format+"\n", args...)
	os.Exit(1)
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

func defaultHome() (string, error) {
	home, err := os.Getwd()
	if err != nil {
		return "", err
	}
	if envHome := os.Getenv("MISTERIO_HOME"); envHome != "" {
		home = envHome
	}
	return home, nil
}
