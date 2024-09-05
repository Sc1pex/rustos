package app

import (
	"bytes"
	"encoding/binary"
	"fmt"
	"os"
	"slices"

	"go.bug.st/serial"
)

type Upload struct{}

func (r Upload) sendInput() bool { return false }

func (u Upload) run(state AppState) (State, error) {
	kernel, err := os.ReadFile(state.kernel_path)
	if err != nil {
		return nil, err
	}

	uploadBytes(kernel, state)

	return Relay{}, nil
}

func uploadBytes(kernel []byte, state AppState) error {
	size := uint32(len(kernel))

	buf := new(bytes.Buffer)
	err := binary.Write(buf, binary.LittleEndian, size)
	if err != nil {
		return err
	}

	state.port.Write(buf.Bytes())

	const blockSize = 512
	blocks := size / blockSize
	if size%blockSize != 0 {
		blocks += 1
	}

	b := 1
	for block := range slices.Chunk(kernel, blockSize) {
		for {
			err := writeAll(state.port, block)
			if err != nil {
				return err
			}

			rbuf := make([]byte, len(block))
			err = readAll(state.port, rbuf)
			if err != nil {
				return err
			}

			if !bytes.Equal(rbuf, block) {
				fmt.Printf("Block %v was wrong. Trying again\r\n", b)

				err := writeAll(state.port, []byte{'B'})
				if err != nil {
					return err
				}
			} else {
				err := writeAll(state.port, []byte{'G'})
				if err != nil {
					return err
				}
				break
			}
		}
		fmt.Printf("Wrote block %v/%v\r", b+1, blocks)
		b += 1
	}

	return nil
}

func writeAll(port serial.Port, b []byte) error {
	start := 0
	for start < len(b) {
		n, err := port.Write(b[start:])
		if err != nil {
			return err
		}
		start += n
	}
	return nil
}

func readAll(port serial.Port, b []byte) error {
	start := 0
	for start < len(b) {
		n, err := port.Read(b[start:])
		if err != nil {
			return err
		}
		start += n
	}

	return nil
}
