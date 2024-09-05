package cli

import "errors"

type Args struct {
	Kernel_path   string
	Serial_device string
}

func ParseCli(os_args []string) (*Args, error) {
	args := Args{Kernel_path: "", Serial_device: "/dev/ttyUSB0"}

	// First argument is the binary path so skip it
	i := 1
	for i < len(os_args) {
		if os_args[i] == "-d" {
			i += 1
			if i >= len(os_args) {
				return nil, errors.New("Expected serial device name after -d")
			}
			args.Serial_device = os_args[i]
		} else {
			args.Kernel_path = os_args[i]
		}

		i += 1
	}

	if args.Kernel_path == "" {
		return nil, errors.New("Expected kernel file")
	}

	return &args, nil
}
