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
		default:
			positional = append(positional, arg)
		}
	}

	if len(positional) != 3 {
		printUsage()
		os.Exit(2)
	}

	manager := misterio.NewManager(home, os.Stdout)
	if err := manager.MoveRole(positional[0], positional[1], positional[2]); err != nil {
		exitf("%v", err)
	}
}

func printUsage() {
	fmt.Println("Usage: misterio-mv [--home PATH] ROLE SOURCE_HOST DESTINATION_HOST")
}

func exitf(format string, args ...any) {
	fmt.Fprintf(os.Stderr, format+"\n", args...)
	os.Exit(1)
}
