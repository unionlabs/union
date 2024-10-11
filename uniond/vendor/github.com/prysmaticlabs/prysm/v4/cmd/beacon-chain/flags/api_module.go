package flags

import "strings"

const PrysmAPIModule string = "prysm"
const EthAPIModule string = "eth"

func EnableHTTPPrysmAPI(httpModules string) bool {
	return enableAPI(httpModules, PrysmAPIModule)
}

func EnableHTTPEthAPI(httpModules string) bool {
	return enableAPI(httpModules, EthAPIModule)
}

func enableAPI(httpModules, api string) bool {
	for _, m := range strings.Split(httpModules, ",") {
		if strings.EqualFold(m, api) {
			return true
		}
	}
	return false
}
