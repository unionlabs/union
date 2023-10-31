/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://www.w3.org/TR/SVG2/
 *
 * Copyright © 2012 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */

interface SVGPreserveAspectRatio {

  // Alignment Types
  const unsigned short SVG_PRESERVEASPECTRATIO_UNKNOWN = 0;
  const unsigned short SVG_PRESERVEASPECTRATIO_NONE = 1;
  const unsigned short SVG_PRESERVEASPECTRATIO_XMINYMIN = 2;
  const unsigned short SVG_PRESERVEASPECTRATIO_XMIDYMIN = 3;
  const unsigned short SVG_PRESERVEASPECTRATIO_XMAXYMIN = 4;
  const unsigned short SVG_PRESERVEASPECTRATIO_XMINYMID = 5;
  const unsigned short SVG_PRESERVEASPECTRATIO_XMIDYMID = 6;
  const unsigned short SVG_PRESERVEASPECTRATIO_XMAXYMID = 7;
  const unsigned short SVG_PRESERVEASPECTRATIO_XMINYMAX = 8;
  const unsigned short SVG_PRESERVEASPECTRATIO_XMIDYMAX = 9;
  const unsigned short SVG_PRESERVEASPECTRATIO_XMAXYMAX = 10;

  // Meet-or-slice Types
  const unsigned short SVG_MEETORSLICE_UNKNOWN = 0;
  const unsigned short SVG_MEETORSLICE_MEET = 1;
  const unsigned short SVG_MEETORSLICE_SLICE = 2;

  [SetterThrows]
  attribute unsigned short align;
  [SetterThrows]
  attribute unsigned short meetOrSlice;
};

