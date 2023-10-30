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

interface SVGMatrix {

  [SetterThrows]
  attribute float a;
  [SetterThrows]
  attribute float b;
  [SetterThrows]
  attribute float c;
  [SetterThrows]
  attribute float d;
  [SetterThrows]
  attribute float e;
  [SetterThrows]
  attribute float f;

  [NewObject]
  SVGMatrix multiply(SVGMatrix secondMatrix);
  [NewObject, Throws]
  SVGMatrix inverse();
  [NewObject]
  SVGMatrix translate(float x, float y);
  [NewObject]
  SVGMatrix scale(float scaleFactor);
  [NewObject]
  SVGMatrix scaleNonUniform(float scaleFactorX, float scaleFactorY);
  [NewObject]
  SVGMatrix rotate(float angle);
  [NewObject, Throws]
  SVGMatrix rotateFromVector(float x, float y);
  [NewObject]
  SVGMatrix flipX();
  [NewObject]
  SVGMatrix flipY();
  [NewObject, Throws]
  SVGMatrix skewX(float angle);
  [NewObject, Throws]
  SVGMatrix skewY(float angle);
};

