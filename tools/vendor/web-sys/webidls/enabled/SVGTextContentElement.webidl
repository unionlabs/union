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

interface SVGTextContentElement : SVGGraphicsElement {

  // lengthAdjust Types
  const unsigned short LENGTHADJUST_UNKNOWN = 0;
  const unsigned short LENGTHADJUST_SPACING = 1;
  const unsigned short LENGTHADJUST_SPACINGANDGLYPHS = 2;

  [Constant]
  readonly attribute SVGAnimatedLength textLength;
  [Constant]
  readonly attribute SVGAnimatedEnumeration lengthAdjust;

  long getNumberOfChars();
  float getComputedTextLength();
  [Throws]
  float getSubStringLength(unsigned long charnum, unsigned long nchars);
  [Throws]
  SVGPoint getStartPositionOfChar(unsigned long charnum);
  [Throws]
  SVGPoint getEndPositionOfChar(unsigned long charnum);
  [NewObject, Throws]
  SVGRect getExtentOfChar(unsigned long charnum);
  [Throws]
  float getRotationOfChar(unsigned long charnum);
  long getCharNumAtPosition(SVGPoint point);
  [Throws]
  undefined selectSubString(unsigned long charnum, unsigned long nchars);
};


