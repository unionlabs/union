/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://dev.w3.org/csswg/css-font-loading/#FontFaceSet-interface
 *
 * Copyright © 2014 W3C® (MIT, ERCIM, Keio, Beihang), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */

dictionary FontFaceSetLoadEventInit : EventInit {
  sequence<FontFace> fontfaces = [];
};

[Constructor(DOMString type, optional FontFaceSetLoadEventInit eventInitDict),
 Pref="layout.css.font-loading-api.enabled"]
interface FontFaceSetLoadEvent : Event {
  [Cached, Constant, Frozen] readonly attribute sequence<FontFace> fontfaces;
};
