package app

import (
	"bytes"
	"encoding/binary"
	"fmt"
	"os"
	"serialpush/encode"
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

	err = uploadBytes(kernel, state)
	if err != nil {
		return nil, err
	}

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

	const blockSize = 1024
	blocks := size / blockSize
	if size%blockSize != 0 {
		blocks += 1
	}

	b := 1
	for block := range slices.Chunk(kernel, blockSize) {
		block_rle := encode.EncodeRLE(block)
		chunk_kind := byte('N')
		// fmt.Printf("Normal: %d, RLE: %d\r\n", len(block), len(block_rle))
		if len(block_rle) < len(block) {
			chunk_kind = byte('R')
			block = block_rle
		}
		// fmt.Printf("Using %s, (%d)\r\n", string(chunk_kind), len(block))

		for {
			err := writeAll(state.port, []byte{chunk_kind})
			if err != nil {
				return err
			}

			buf := new(bytes.Buffer)
			err = binary.Write(buf, binary.LittleEndian, uint32(len(block)))
			if err != nil {
				return err
			}
			err = writeAll(state.port, buf.Bytes())
			if err != nil {
				return err
			}
			// fmt.Printf("%v\r\n", buf.Bytes())
			// fmt.Printf("Wrote kind and length\r\n")

			err = writeAll(state.port, block)
			if err != nil {
				return err
			}
			// fmt.Printf("Wrote block\r\n")

			rbuf := make([]byte, 5+len(block))
			err = readAll(state.port, rbuf)
			if err != nil {
				return err
			}
			// fmt.Printf("Read data: %v\r\n", rbuf)

			same_chunk := rbuf[0] == chunk_kind
			same_len := binary.LittleEndian.Uint32(rbuf[1:5]) == uint32(len(block))
			same_data := bytes.Equal(rbuf[5:], block)
			if !same_chunk || !same_len || !same_data {
				fmt.Printf("Block %v was wrong (chunk: %v len: %v data: %v). Trying again\r\n", b, same_chunk, same_len, same_data)

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
		fmt.Printf("Wrote block %v/%v\r", b, blocks)
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
