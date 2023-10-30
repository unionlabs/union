/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://dvcs.w3.org/hg/webcomponents/raw-file/tip/spec/shadow/index.html
 *
 * Copyright © 2012 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */

// https://dom.spec.whatwg.org/#enumdef-shadowrootmode
enum ShadowRootMode {
  "open",
  "closed"
};

// https://dom.spec.whatwg.org/#shadowroot
[Func="nsDocument::IsShadowDOMEnabled"]
interface ShadowRoot : DocumentFragment
{
  // Shadow DOM v1
  readonly attribute ShadowRootMode mode;
  readonly attribute Element host;

  // [deprecated] Shadow DOM v0
  Element? getElementById(DOMString elementId);
  HTMLCollection getElementsByTagName(DOMString localName);
  HTMLCollection getElementsByTagNameNS(DOMString? namespace, DOMString localName);
  HTMLCollection getElementsByClassName(DOMString classNames);
  [CEReactions, SetterThrows, TreatNullAs=EmptyString]
  attribute DOMString innerHTML;
};

ShadowRoot includes DocumentOrShadowRoot;
