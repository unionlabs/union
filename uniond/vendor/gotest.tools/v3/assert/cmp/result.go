package cmp

import (
	"bytes"
	"fmt"
	"go/ast"
	"reflect"
	"text/template"

	"gotest.tools/v3/internal/source"
)

// A Result of a [Comparison].
type Result interface {
	Success() bool
}

// StringResult is an implementation of [Result] that reports the error message
// string verbatim and does not provide any templating or formatting of the
// message.
type StringResult struct {
	success bool
	message string
}

// Success returns true if the comparison was successful.
func (r StringResult) Success() bool {
	return r.success
}

// FailureMessage returns the message used to provide additional information
// about the failure.
func (r StringResult) FailureMessage() string {
	return r.message
}

// ResultSuccess is a constant which is returned by a [Comparison] to
// indicate success.
var ResultSuccess = StringResult{success: true}

// ResultFailure returns a failed [Result] with a failure message.
func ResultFailure(message string) StringResult {
	return StringResult{message: message}
}

// ResultFromError returns [ResultSuccess] if err is nil. Otherwise [ResultFailure]
// is returned with the error message as the failure message.
func ResultFromError(err error) Result {
	if err == nil {
		return ResultSuccess
	}
	return ResultFailure(err.Error())
}

type templatedResult struct {
	template string
	data     map[string]interface{}
}

func (r templatedResult) Success() bool {
	return false
}

func (r templatedResult) FailureMessage(args []ast.Expr) string {
	msg, err := renderMessage(r, args)
	if err != nil {
		return fmt.Sprintf("failed to render failure message: %s", err)
	}
	return msg
}

func (r templatedResult) UpdatedExpected(stackIndex int) error {
	// TODO: would be nice to have structured data instead of a map
	return source.UpdateExpectedValue(stackIndex+1, r.data["x"], r.data["y"])
}

// ResultFailureTemplate returns a [Result] with a template string and data which
// can be used to format a failure message. The template may access data from .Data,
// the comparison args with the callArg function, and the formatNode function may
// be used to format the call args.
func ResultFailureTemplate(template string, data map[string]interface{}) Result {
	return templatedResult{template: template, data: data}
}

func renderMessage(result templatedResult, args []ast.Expr) (string, error) {
	tmpl := template.New("failure").Funcs(template.FuncMap{
		"formatNode": source.FormatNode,
		"callArg": func(index int) ast.Expr {
			if index >= len(args) {
				return nil
			}
			return args[index]
		},
		// TODO: any way to include this from ErrorIS instead of here?
		"notStdlibErrorType": func(typ interface{}) bool {
			r := reflect.TypeOf(typ)
			return r != stdlibFmtErrorType && r != stdlibErrorNewType
		},
	})
	var err error
	tmpl, err = tmpl.Parse(result.template)
	if err != nil {
		return "", err
	}
	buf := new(bytes.Buffer)
	err = tmpl.Execute(buf, map[string]interface{}{
		"Data": result.data,
	})
	return buf.String(), err
}
