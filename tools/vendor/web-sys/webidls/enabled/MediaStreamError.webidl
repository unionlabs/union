/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://w3c.github.io/mediacapture-main/getusermedia.html#idl-def-MediaStreamError
 */

// The future of MediaStreamError is uncertain.
// https://www.w3.org/Bugs/Public/show_bug.cgi?id=26776

// TODO: This is an 'exception', not an interface, by virtue of needing to be
// passed as a promise rejection-reason. Revisit if DOMException grows a customArg

[ExceptionClass, NoInterfaceObject]
interface MediaStreamError {
  readonly attribute DOMString  name;
  readonly attribute DOMString? message;
  readonly attribute DOMString? constraint;
};
