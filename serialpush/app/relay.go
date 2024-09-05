package app

import (
	"fmt"
	"strings"
)

type StateOutput struct {
	newState State
	err      error
}

func (r Relay) run(state AppState) (State, error) {
	var zerocnt int = 0
	buf := make([]byte, 128)
	for {
		n, err := state.port.Read(buf)
		if err != nil {
			return nil, err
		}

		for i, b := range buf[:n] {
			if b == 0 {
				zerocnt += 1
			} else {
				zerocnt = 0
			}
			if zerocnt == 3 {
				s := string(buf[:i-2])
				s = strings.ReplaceAll(s, "\n", "\r\n")
				fmt.Printf("%v", s)
				break
			}
		}
		if zerocnt == 3 {
			return Upload{}, nil
		} else {
			s := string(buf[:n])
			s = strings.ReplaceAll(s, "\n", "\r\n")
			fmt.Printf("%v", s)
		}
	}
}

type Relay struct{}

func (r Relay) sendInput() bool { return true }
