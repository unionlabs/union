package encoding

func (e *encoder) setByte1Int64(value int64) error {
	return e.buf.Write(e.w, byte(value))
}

func (e *encoder) setByte2Int64(value int64) error {
	return e.buf.Write(e.w,
		byte(value>>8),
		byte(value),
	)
}

func (e *encoder) setByte4Int64(value int64) error {
	return e.buf.Write(e.w,
		byte(value>>24),
		byte(value>>16),
		byte(value>>8),
		byte(value),
	)
}

func (e *encoder) setByte8Int64(value int64) error {
	return e.buf.Write(e.w,
		byte(value>>56),
		byte(value>>48),
		byte(value>>40),
		byte(value>>32),
		byte(value>>24),
		byte(value>>16),
		byte(value>>8),
		byte(value),
	)
}

func (e *encoder) setByte1Uint64(value uint64) error {
	return e.buf.Write(e.w, byte(value))
}

func (e *encoder) setByte2Uint64(value uint64) error {
	return e.buf.Write(e.w,
		byte(value>>8),
		byte(value),
	)
}

func (e *encoder) setByte4Uint64(value uint64) error {
	return e.buf.Write(e.w,
		byte(value>>24),
		byte(value>>16),
		byte(value>>8),
		byte(value),
	)
}

func (e *encoder) setByte8Uint64(value uint64) error {
	return e.buf.Write(e.w,
		byte(value>>56),
		byte(value>>48),
		byte(value>>40),
		byte(value>>32),
		byte(value>>24),
		byte(value>>16),
		byte(value>>8),
		byte(value),
	)
}

func (e *encoder) setByte1Int(value int) error {
	return e.buf.Write(e.w,
		byte(value),
	)
}

func (e *encoder) setByte2Int(value int) error {
	return e.buf.Write(e.w,
		byte(value>>8),
		byte(value),
	)
}

func (e *encoder) setByte4Int(value int) error {
	return e.buf.Write(e.w,
		byte(value>>24),
		byte(value>>16),
		byte(value>>8),
		byte(value),
	)
}

func (e *encoder) setBytes(bs []byte) error {
	return e.buf.Write(e.w, bs...)
}
