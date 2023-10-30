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

[Constructor(optional DOMString data = "")]
interface Text : CharacterData {
  [Throws]
  Text splitText(unsigned long offset);
  [Throws]
  readonly attribute DOMString wholeText;
};

partial interface Text {
  [BinaryName="assignedSlotByMode", Func="nsTextNode::IsShadowDOMEnabled"]
  readonly attribute HTMLSlotElement? assignedSlot;
};

Text includes GeometryUtils;
