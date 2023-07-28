package levigo

// #include "leveldb/c.h"
import "C"

func boolToUchar(b bool) C.uchar {
	uc := C.uchar(0)
	if b {
		uc = C.uchar(1)
	}
	return uc
}

func ucharToBool(uc C.uchar) bool {
	if uc == C.uchar(0) {
		return false
	}
	return true
}
