package jsonrpc

import "testing"

func TestError(t *testing.T) {
	wantCode := ParseError
	sut := Error{
		Code: wantCode,
	}

	gotCode := sut.ErrorCode()
	if gotCode != wantCode {
		t.Fatalf("want=%d, got=%d", gotCode, wantCode)
	}

	if sut.Error() == "" {
		t.Fatal("Empty error string.")
	}

	want := "override"
	sut.Message = want
	got := sut.Error()
	if sut.Error() != want {
		t.Fatalf("overridden error message: want=%s, got=%s", want, got)
	}

}
func TestErrorsSatisfyError(t *testing.T) {
	errs := []interface{}{
		parseError("parseError"),
		invalidRequestError("invalidRequestError"),
		methodNotFoundError("methodNotFoundError"),
		invalidParamsError("invalidParamsError"),
		internalError("internalError"),
	}
	for _, e := range errs {
		err, ok := e.(error)
		if !ok {
			t.Fatalf("Couldn't assert %s as error.", e)
		}
		errString := err.Error()
		if errString == "" {
			t.Fatal("Empty error string")
		}

		ec, ok := e.(ErrorCoder)
		if !ok {
			t.Fatalf("Couldn't assert %s as ErrorCoder.", e)
		}
		if ErrorMessage(ec.ErrorCode()) == "" {
			t.Fatalf("Error type %s returned code of %d, which does not map to error string", e, ec.ErrorCode())
		}
	}
}
