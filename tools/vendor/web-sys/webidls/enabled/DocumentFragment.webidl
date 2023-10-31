/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://www.w3.org/TR/2012/WD-dom-20120405/#interface-documentfragment
 * http://www.w3.org/TR/2012/WD-selectors-api-20120628/#interface-definitions
 *
 * Copyright © 2012 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */

[Constructor]
interface DocumentFragment : Node {
  Element? getElementById(DOMString elementId);
};

// http://www.w3.org/TR/2012/WD-selectors-api-20120628/#interface-definitions
partial interface DocumentFragment {
  [Throws]
  Element?  querySelector(DOMString selectors);
  [Throws]
  NodeList  querySelectorAll(DOMString selectors);
};

DocumentFragment includes ParentNode;
