/*
Package errors implements custom error interfaces for cosmos-sdk.

Error declarations should be generic and cover broad range of cases. Each
returned error instance can wrap a generic error declaration to provide more
details.

This package provides a broad range of errors declared that fits all common
cases. If an error is very specific for an extension it can be registered outside
of the errors package. If it will be needed my many extensions, please consider
registering it in the errors package. To create a new error instance use Register
function. You must provide a unique, non zero error code and a short description, for example:

  var ErrZeroDivision = errors.Register(9241, "zero division")

When returning an error, you can attach to it an additional context
information by using Wrap function, for example:

   func safeDiv(val, div int) (int, err) {
	   if div == 0 {
		   return 0, errors.Wrapf(ErrZeroDivision, "cannot divide %d", val)
	   }
	   return val / div, nil
   }

The first time an error instance is wrapped a stacktrace is attached as well.
Stacktrace information can be printed using %+v and %v formats.

  %s  is just the error message
  %+v is the full stack trace
  %v  appends a compressed [filename:line] where the error was created

*/
package errors
