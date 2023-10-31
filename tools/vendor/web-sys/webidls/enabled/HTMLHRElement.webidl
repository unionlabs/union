/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://www.whatwg.org/specs/web-apps/current-work/#the-hr-element
 * http://www.whatwg.org/specs/web-apps/current-work/#other-elements,-attributes-and-apis
 * © Copyright 2004-2011 Apple Computer, Inc., Mozilla Foundation, and
 * Opera Software ASA. You are granted a license to use, reproduce
 * and create derivative works of this document.
 */

// http://www.whatwg.org/specs/web-apps/current-work/#the-hr-element
[HTMLConstructor]
interface HTMLHRElement : HTMLElement {
};

// http://www.whatwg.org/specs/web-apps/current-work/#other-elements,-attributes-and-apis
partial interface HTMLHRElement {
           [CEReactions, SetterThrows]
           attribute DOMString align;
           [CEReactions, SetterThrows]
           attribute DOMString color;
           [CEReactions, SetterThrows]
           attribute boolean noShade;
           [CEReactions, SetterThrows]
           attribute DOMString size;
           [CEReactions, SetterThrows]
           attribute DOMString width;
};
