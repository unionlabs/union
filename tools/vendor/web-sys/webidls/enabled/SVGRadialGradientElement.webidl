/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://svgwg.org/svg2-draft/
 *
 * Copyright © 2012 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */

interface SVGRadialGradientElement : SVGGradientElement {
  [Constant]
  readonly attribute SVGAnimatedLength cx;
  [Constant]
  readonly attribute SVGAnimatedLength cy;
  [Constant]
  readonly attribute SVGAnimatedLength r;
  [Constant]
  readonly attribute SVGAnimatedLength fx;
  [Constant]
  readonly attribute SVGAnimatedLength fy;
  // XXX: Bug 1242048
  // [SameObject]
  readonly attribute SVGAnimatedLength fr;
};
