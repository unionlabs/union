/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://www.w3.org/TR/2012/WD-dom-20120105/
 *
 * Copyright © 2012 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */


dictionary EventListenerOptions {
  boolean capture = false;
};

dictionary AddEventListenerOptions : EventListenerOptions {
  boolean passive;
  boolean once = false;
};

[Constructor,
 Exposed=(Window,Worker,WorkerDebugger,AudioWorklet,System)]
interface EventTarget {
  /* Passing null for wantsUntrusted means "default behavior", which
     differs in content and chrome.  In content that default boolean
     value is true, while in chrome the default boolean value is
     false. */
  [Throws]
  undefined addEventListener(DOMString type,
                        EventListener listener,
                        optional (AddEventListenerOptions or boolean) options,
                        optional boolean? wantsUntrusted = null);
  [Throws]
  undefined removeEventListener(DOMString type,
                           EventListener listener,
                           optional (EventListenerOptions or boolean) options);
  [Throws, NeedsCallerType]
  boolean dispatchEvent(Event event);
};
