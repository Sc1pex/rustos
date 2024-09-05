package encode_test

import (
	"serialpush/encode"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestSimple(t *testing.T) {
	input := []byte{1, 1, 1, 1, 1, 2, 2, 3, 4, 4}
	output := encode.EncodeRLE(input)
	out_exp := []byte{5, 1, 2, 2, 1, 3, 2, 4}

	assert.Equal(t, output, out_exp)
}

func TestLong(t *testing.T) {
	input := make([]byte, 255)
	for i := range input {
		input[i] = 1
	}
	output := encode.EncodeRLE(input)
	out_exp := []byte{255, 1}

	assert.Equal(t, output, out_exp)
}

func TestLong1(t *testing.T) {
	input := make([]byte, 500)
	for i := range input {
		input[i] = 1
	}
	output := encode.EncodeRLE(input)
	out_exp := []byte{255, 1, 245, 1}

	assert.Equal(t, output, out_exp)
}
