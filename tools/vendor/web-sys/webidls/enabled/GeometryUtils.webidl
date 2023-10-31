/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://dev.w3.org/csswg/cssom-view/
 *
 * Copyright © 2012 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */

enum CSSBoxType { "margin", "border", "padding", "content" };
dictionary BoxQuadOptions {
  CSSBoxType box = "border";
  GeometryNode relativeTo;
};

dictionary ConvertCoordinateOptions {
  CSSBoxType fromBox = "border";
  CSSBoxType toBox = "border";
};

interface mixin GeometryUtils {
  [Throws, Func="nsINode::HasBoxQuadsSupport", NeedsCallerType]
  sequence<DOMQuad> getBoxQuads(optional BoxQuadOptions options);
  [Throws, Pref="layout.css.convertFromNode.enabled", NeedsCallerType]
  DOMQuad convertQuadFromNode(DOMQuad quad, GeometryNode from, optional ConvertCoordinateOptions options);
  [Throws, Pref="layout.css.convertFromNode.enabled", NeedsCallerType]
  DOMQuad convertRectFromNode(DOMRectReadOnly rect, GeometryNode from, optional ConvertCoordinateOptions options);
  [Throws, Pref="layout.css.convertFromNode.enabled", NeedsCallerType]
  DOMPoint convertPointFromNode(DOMPointInit point, GeometryNode from, optional ConvertCoordinateOptions options);
};

// PseudoElement includes GeometryUtils;

typedef (Text or Element /* or PseudoElement */ or Document) GeometryNode;
