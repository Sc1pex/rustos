package app

import (
	"fmt"
	"os"
	"serialpush/cli"

	"go.bug.st/serial"
	"golang.org/x/term"
)

type AppState struct {
	port        serial.Port
	kernel_path string
}

type App struct {
	oldTermState *term.State
	appState     AppState

	current State
}

func InitApp(args cli.Args) (*App, error) {
	mode := &serial.Mode{
		BaudRate: 921600,
	}
	port, err := serial.Open(args.Serial_device, mode)
	if err != nil {
		return nil, err
	}

	oldTermState, err := term.MakeRaw(int(os.Stdin.Fd()))
	if err != nil {
		return nil, err
	}

	return &App{
		oldTermState,
		AppState{
			port,
			args.Kernel_path,
		},
		Relay{},
	}, nil
}

func (app *App) term_input() {
	var c rune
	for {
		_, err := fmt.Scanf("%c", &c)
		if err != nil {
			fmt.Fprintf(os.Stderr, "Failed to read: %s", err)
			term.Restore(int(os.Stdin.Fd()), app.oldTermState)
			os.Exit(1)
		}
		// Ctrl-c or Ctrl-d
		if int(c) == 3 || int(c) == 4 {
			term.Restore(int(os.Stdin.Fd()), app.oldTermState)
			os.Exit(0)
		}
		// Ctrl-Alt + something
		if int(c) == 27 {
			continue
		}
		// Ctrl-l
		if int(c) == 12 {
			fmt.Printf("\033[2J\033[H")
		}

		if app.current.sendInput() {
			app.appState.port.Write([]byte(string(c)))
		}
	}
}

func (app *App) Run() error {
	defer term.Restore(int(os.Stdin.Fd()), app.oldTermState)
	go app.term_input()

	var err error
	for {
		app.current, err = app.current.run(app.appState)
		if err != nil {
			return err
		}
		if app.current == nil {
			return nil
		}
	}
}

type State interface {
	run(AppState) (State, error)
	sendInput() bool
}
