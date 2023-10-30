/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://www.w3.org/TR/2012/WD-html5-20120329/
 *
 * Copyright © 2012 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */

interface HTMLOptionsCollection : HTMLCollection {
  [CEReactions, SetterThrows]
  attribute unsigned long length;
  [CEReactions, Throws]
  setter undefined (unsigned long index, HTMLOptionElement? option);
  [CEReactions, Throws]
  undefined add((HTMLOptionElement or HTMLOptGroupElement) element, optional (HTMLElement or long)? before = null);
  [CEReactions, Throws]
  undefined remove(long index);
  [Throws]
  attribute long selectedIndex;
};
