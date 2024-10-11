package httputil

import (
	"net/http"
)

func HandleError(w http.ResponseWriter, message string, code int) {
	errJson := &DefaultJsonError{
		Message: message,
		Code:    code,
	}
	WriteError(w, errJson)
}
