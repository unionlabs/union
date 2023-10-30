/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://drafts.csswg.org/css-pseudo/#CSSPseudoElement-interface
 * https://drafts.csswg.org/cssom/#pseudoelement
 *
 * Copyright © 2015 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */

// Both CSSOM and CSS Pseudo-Elements 4 provide contradictory definitions for
// this interface.
// What we implement here is a minimal subset of the two definitions which we
// ship behind a pref until the specification issues have been resolved.
[Func="nsDocument::IsWebAnimationsEnabled"]
interface CSSPseudoElement {
  readonly attribute DOMString type;
  readonly attribute Element parentElement;
};

// https://drafts.csswg.org/web-animations/#extensions-to-the-pseudoelement-interface
CSSPseudoElement includes Animatable;
