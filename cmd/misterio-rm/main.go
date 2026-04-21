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
	showHelp := false

	fs := flag.NewFlagSet("misterio-rm", flag.ContinueOnError)
	fs.SetOutput(os.Stderr)
	fs.Usage = printUsage
	fs.StringVar(&home, "home", home, "")
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
	if err := manager.RemoveRoles(positional[0], positional[1:]); err != nil {
		exitf("%v", err)
	}
}

func printUsage() {
	fmt.Println("Usage: misterio-rm [--home PATH] SOURCE_HOST ROLE [ROLE...]")
}

func exitf(format string, args ...any) {
	fmt.Fprintf(os.Stderr, format+"\n", args...)
	os.Exit(1)
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
