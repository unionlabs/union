package decoding

func (d *decoder) readSize1() (byte, error) {
	if _, err := d.r.Read(d.buf.B1); err != nil {
		return 0, err
	}
	return d.buf.B1[0], nil
}

func (d *decoder) readSize2() ([]byte, error) {
	if _, err := d.r.Read(d.buf.B2); err != nil {
		return emptyBytes, err
	}
	return d.buf.B2, nil
}

func (d *decoder) readSize4() ([]byte, error) {
	if _, err := d.r.Read(d.buf.B4); err != nil {
		return emptyBytes, err
	}
	return d.buf.B4, nil
}

func (d *decoder) readSize8() ([]byte, error) {
	if _, err := d.r.Read(d.buf.B8); err != nil {
		return emptyBytes, err
	}
	return d.buf.B8, nil
}

func (d *decoder) readSize16() ([]byte, error) {
	if _, err := d.r.Read(d.buf.B16); err != nil {
		return emptyBytes, err
	}
	return d.buf.B16, nil
}

func (d *decoder) readSizeN(n int) ([]byte, error) {
	var b []byte
	if n <= len(d.buf.Data) {
		b = d.buf.Data[:n]
	} else {
		d.buf.Data = append(d.buf.Data, make([]byte, n-len(d.buf.Data))...)
		b = d.buf.Data
	}
	if _, err := d.r.Read(b); err != nil {
		return emptyBytes, err
	}
	return b, nil
}
