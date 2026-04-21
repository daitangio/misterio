If you mean **third-party Go libraries**, the 3 most used are probably:

1. `github.com/spf13/cobra`
   Very widely used for full CLI apps with subcommands.
   `pkg.go.dev` shows about **195,884 importers**.
   Source: https://pkg.go.dev/github.com/spf13/cobra

2. `github.com/spf13/pflag`
   Common for GNU-style flags like `--port` and `-p`; often used under Cobra.
   `pkg.go.dev` shows about **54,971 importers**.
   Source: https://pkg.go.dev/github.com/spf13/pflag

3. `github.com/urfave/cli`
   Popular alternative for building CLI apps with commands and flags.
   `pkg.go.dev` shows about **20,357 importers**.
   Source: https://pkg.go.dev/github.com/urfave/cli

If you include the **standard library**, then `flag` is the default baseline and is arguably the most used overall:
https://pkg.go.dev/flag

Practical advice:
- Use `flag` for small tools.
- Use `cobra` for larger CLIs with subcommands.
- Use `urfave/cli` if you want something simpler than Cobra.
