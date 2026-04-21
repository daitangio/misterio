package main

import (
	"fmt"
	"os"
	"strings"

	"github.com/daitangio/misterio/internal/misterio"
)

func main() {
	home, err := os.Getwd()
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
	if envHome := os.Getenv("MISTERIO_HOME"); envHome != "" {
		home = envHome
	}

	build := false
	var positional []string

	args := os.Args[1:]
	for i := 0; i < len(args); i++ {
		arg := args[i]
		switch {
		case arg == "--help":
			printUsage()
			return
		case arg == "--home":
			i++
			if i >= len(args) {
				exitf("missing value for --home")
			}
			home = args[i]
		case strings.HasPrefix(arg, "--home="):
			home = strings.TrimPrefix(arg, "--home=")
		case arg == "--build":
			build = true
		case arg == "--no-build":
			build = false
		default:
			positional = append(positional, arg)
		}
	}

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
