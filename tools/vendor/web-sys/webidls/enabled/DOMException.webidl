/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://dom.spec.whatwg.org/#exception-domexception
 *
 * Copyright © 2012 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */


// This is the WebIDL version of nsIException.  This is mostly legacy stuff.

// invalid widl
//interface StackFrame;

[Exposed=(Window,Worker,System)]
interface mixin ExceptionMembers
{
  // The nsresult associated with this exception.
  readonly attribute unsigned long           result;

  // Filename location.  This is the location that caused the
  // error, which may or may not be a source file location.
  // For example, standard language errors would generally have
  // the same location as their top stack entry.  File
  // parsers may put the location of the file they were parsing,
  // etc.

  // null indicates "no data"
  readonly attribute DOMString               filename;
  // Valid line numbers begin at '1'. '0' indicates unknown.
  readonly attribute unsigned long           lineNumber;
  // Valid column numbers begin at 0.
  // We don't have an unambiguous indicator for unknown.
  readonly attribute unsigned long           columnNumber;

  // A stack trace, if available.  nsIStackFrame does not have classinfo so
  // this was only ever usefully available to chrome JS.
  [ChromeOnly, Exposed=Window]
  readonly attribute StackFrame?             location;

  // Arbitary data for the implementation.
  [Exposed=Window]
  readonly attribute nsISupports?            data;

  // Formatted exception stack
  [Replaceable]
  readonly attribute DOMString               stack;
};

[NoInterfaceObject, Exposed=(Window,Worker)]
interface Exception {
  // The name of the error code (ie, a string repr of |result|).
  readonly attribute DOMString               name;
  // A custom message set by the thrower.
  readonly attribute DOMString               message;
  // A generic formatter - make it suitable to print, etc.
  stringifier;
};

Exception includes ExceptionMembers;

// XXXkhuey this is an 'exception', not an interface, but we don't have any
// parser or codegen mechanisms for dealing with exceptions.
[ExceptionClass,
 Exposed=(Window, Worker,System),
 Constructor(optional DOMString message = "", optional DOMString name)]
interface DOMException {
  // The name of the error code (ie, a string repr of |result|).
  readonly attribute DOMString               name;
  // A custom message set by the thrower.
  readonly attribute DOMString               message;
  readonly attribute unsigned short code;

  const unsigned short INDEX_SIZE_ERR = 1;
  const unsigned short DOMSTRING_SIZE_ERR = 2; // historical
  const unsigned short HIERARCHY_REQUEST_ERR = 3;
  const unsigned short WRONG_DOCUMENT_ERR = 4;
  const unsigned short INVALID_CHARACTER_ERR = 5;
  const unsigned short NO_DATA_ALLOWED_ERR = 6; // historical
  const unsigned short NO_MODIFICATION_ALLOWED_ERR = 7;
  const unsigned short NOT_FOUND_ERR = 8;
  const unsigned short NOT_SUPPORTED_ERR = 9;
  const unsigned short INUSE_ATTRIBUTE_ERR = 10; // historical
  const unsigned short INVALID_STATE_ERR = 11;
  const unsigned short SYNTAX_ERR = 12;
  const unsigned short INVALID_MODIFICATION_ERR = 13;
  const unsigned short NAMESPACE_ERR = 14;
  const unsigned short INVALID_ACCESS_ERR = 15;
  const unsigned short VALIDATION_ERR = 16; // historical
  const unsigned short TYPE_MISMATCH_ERR = 17; // historical; use JavaScript's TypeError instead
  const unsigned short SECURITY_ERR = 18;
  const unsigned short NETWORK_ERR = 19;
  const unsigned short ABORT_ERR = 20;
  const unsigned short URL_MISMATCH_ERR = 21;
  const unsigned short QUOTA_EXCEEDED_ERR = 22;
  const unsigned short TIMEOUT_ERR = 23;
  const unsigned short INVALID_NODE_TYPE_ERR = 24;
  const unsigned short DATA_CLONE_ERR = 25;
};

// XXXkhuey copy all of Gecko's non-standard stuff onto DOMException, but leave
// the prototype chain sane.
DOMException includes ExceptionMembers;
