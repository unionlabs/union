/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * This IDL file contains utilities to help connect JS promises to our
 * Web IDL infrastructure.
 */

callback PromiseJobCallback = undefined();

[TreatNonCallableAsNull]
callback AnyCallback = any (any value);

// Hack to allow us to have JS owning and properly tracing/CCing/etc a
// PromiseNativeHandler.
[NoInterfaceObject,
 Exposed=(Window,Worker,System)]
interface PromiseNativeHandler {
};
