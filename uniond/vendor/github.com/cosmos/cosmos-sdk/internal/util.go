package util

import (
	"fmt"
)

func CombineErrors(ret error, also error, desc string) error {
	if also != nil {
		if ret != nil {
			ret = fmt.Errorf("%w; %v: %v", ret, desc, also)
		} else {
			ret = also
		}
	}
	return ret
}
