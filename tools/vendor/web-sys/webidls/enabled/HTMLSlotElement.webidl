/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://html.spec.whatwg.org/multipage/forms.html#the-dialog-element
 *
 * © Copyright 2004-2011 Apple Computer, Inc., Mozilla Foundation, and
 * Opera Software ASA. You are granted a license to use, reproduce
 * and create derivative works of this document.
 */

[Func="nsDocument::IsShadowDOMEnabled", Exposed=Window, HTMLConstructor]
interface HTMLSlotElement : HTMLElement {
  [CEReactions, SetterThrows] attribute DOMString name;
  sequence<Node> assignedNodes(optional AssignedNodesOptions options);
};

dictionary AssignedNodesOptions {
  boolean flatten = false;
};
