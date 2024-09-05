package main

import (
	"fmt"
	"os"

	"serialpush/app"
	"serialpush/cli"
)

func main() {
	args, err := cli.ParseCli(os.Args)
	if err != nil {
		fmt.Fprintf(os.Stderr, "%s\n", err)
		return
	}

	app, err := app.InitApp(*args)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Failed to initialize: %s\n", err)
		return
	}
	err = app.Run()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error: %s\r", err)
	}
}
