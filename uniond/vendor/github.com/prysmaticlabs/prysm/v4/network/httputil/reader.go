package httputil

import (
	"net/http"
	"regexp"
	"strconv"
	"strings"

	"github.com/prysmaticlabs/prysm/v4/api"
)

// match a number with optional decimals
var priorityRegex = regexp.MustCompile(`q=(\d+(?:\.\d+)?)`)

// RespondWithSsz takes a http request and checks to see if it should be requesting a ssz response.
func RespondWithSsz(req *http.Request) bool {
	accept := req.Header.Values("Accept")
	if len(accept) == 0 {
		return false
	}
	types := strings.Split(accept[0], ",")
	currentType, currentPriority := "", 0.0
	for _, t := range types {
		values := strings.Split(t, ";")
		name := values[0]
		if name != api.JsonMediaType && name != api.OctetStreamMediaType {
			continue
		}
		// no params specified
		if len(values) == 1 {
			priority := 1.0
			if priority > currentPriority {
				currentType, currentPriority = name, priority
			}
			continue
		}
		params := values[1]

		match := priorityRegex.FindAllStringSubmatch(params, 1)
		if len(match) != 1 {
			continue
		}
		priority, err := strconv.ParseFloat(match[0][1], 32)
		if err != nil {
			return false
		}
		if priority > currentPriority {
			currentType, currentPriority = name, priority
		}
	}

	return currentType == api.OctetStreamMediaType
}

// IsRequestSsz checks if the request object should be interpreted as ssz
func IsRequestSsz(req *http.Request) bool {
	return req.Header.Get("Content-Type") == api.OctetStreamMediaType
}
