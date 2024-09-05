package encode

func EncodeRLE(data []byte) []byte {
	var result []byte

	cur_byte := data[0]
	cur_len := 1
	for i := 1; i < len(data); i++ {
		if data[i] == cur_byte {
			cur_len++

			if cur_len == 255 {
				result = append(result, byte(cur_len), cur_byte)
				cur_len = 0
			}
		} else {
			result = append(result, byte(cur_len), cur_byte)
			cur_byte = data[i]
			cur_len = 1
		}
	}
	if cur_len != 0 {
		result = append(result, byte(cur_len), cur_byte)
	}

	return result
}
