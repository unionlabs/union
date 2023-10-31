/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.

 * The origin of this IDL file is
 * https://www.khronos.org/registry/webgl/specs/latest/1.0/#fire-a-webgl-context-event
 */

[Constructor(DOMString type, optional WebGLContextEventInit eventInit),
 Exposed=(Window,Worker),
 Func="mozilla::dom::OffscreenCanvas::PrefEnabledOnWorkerThread"]
interface WebGLContextEvent : Event {
  readonly attribute DOMString statusMessage;
};

// EventInit is defined in the DOM4 specification.
dictionary WebGLContextEventInit : EventInit {
  DOMString statusMessage = "";
};
