package main

import (
	"bytes"
	"encoding/binary"
	"fmt"
	"log"
	"os"
	"strings"
	"sync"

	"go.bug.st/serial"
	"golang.org/x/term"
)

var uploading = false

func input(port serial.Port) {
	oldState, err := term.MakeRaw(int(os.Stdin.Fd()))
	if err != nil {
		log.Fatal(err)
		return
	}
	defer term.Restore(int(os.Stdin.Fd()), oldState)

	var c rune
	for {
		_, err := fmt.Scanf("%c", &c)
		if err != nil {
			log.Fatal(err)
			return
		}
		// Ctrl-c or Ctrl-d
		if int(c) == 3 || int(c) == 4 {
			term.Restore(int(os.Stdin.Fd()), oldState)
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

		if uploading {
			continue
		}

		port.Write([]byte(string(c)))
	}
}

func output(port serial.Port, kernelPath string) {
	var zerocnt int = 0
	buf := make([]byte, 128)
	for {
		n, err := port.Read(buf)
		if err != nil {
			log.Fatal(err)
			return
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
			err = uploadKernel(port, kernelPath)
			if err != nil {
				log.Fatal(err)
				return
			}
		} else {
			s := string(buf[:n])
			s = strings.ReplaceAll(s, "\n", "\r\n")
			fmt.Printf("%v", s)
		}

	}
}

func uploadKernel(port serial.Port, kernelPath string) error {
	fmt.Printf("Starting to write kernel\r\n")

	uploading = true
	defer func() { uploading = false }()

	kernel, err := os.ReadFile(kernelPath)
	if err != nil {
		return err
	}

	size := uint32(len(kernel))
	buf := new(bytes.Buffer)
	err = binary.Write(buf, binary.LittleEndian, size)
	if err != nil {
		return err
	}

	port.Write(buf.Bytes())

	blockSize := 512
	blocks := size / uint32(blockSize)

	for b := range blocks {
		for {
			block := kernel[(b * uint32(blockSize)):((b + 1) * uint32(blockSize))]
			err := writeAll(port, block)
			if err != nil {
				return err
			}

			rbuf := make([]byte, blockSize)
			err = readAll(port, rbuf)
			if err != nil {
				return err
			}

			if !bytes.Equal(rbuf, block) {
				fmt.Printf("Block %v was wrong. Trying again\r\n", b)

				err := writeAll(port, []byte{'B'})
				if err != nil {
					return err
				}
			} else {
				err := writeAll(port, []byte{'G'})
				if err != nil {
					return err
				}
				break
			}
		}
		fmt.Printf("Wrote block %v/%v\r", b+1, blocks)
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

func main() {
	log.SetFlags(log.LstdFlags | log.Lshortfile)
	mode := &serial.Mode{
		BaudRate: 921600,
	}
	port, err := serial.Open("/dev/ttyUSB0", mode)
	if err != nil {
		log.Fatal(err)
	}

	kernel := "kernel8.img"
	args := os.Args[1:]
	if len(args) != 1 {
		fmt.Println("Usage: serial <kernel_file>")
	} else {
		kernel = args[0]
	}

	var wg sync.WaitGroup
	wg.Add(1)
	go func() {
		input(port)
		wg.Done()
	}()
	go func() {
		output(port, kernel)
		wg.Done()
	}()
	wg.Wait()
}
