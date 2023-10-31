/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://drafts.csswg.org/web-animations/#the-keyframeeffect-interfaces
 *
 * Copyright © 2015 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */

enum IterationCompositeOperation {
  "replace",
  "accumulate"
};

dictionary KeyframeEffectOptions : EffectTiming {
  IterationCompositeOperation iterationComposite = "replace";
  CompositeOperation          composite = "replace";
};

// KeyframeEffect should run in the caller's compartment to do custom
// processing on the `keyframes` object.
[Func="nsDocument::IsWebAnimationsEnabled",
 RunConstructorInCallerCompartment,
 Constructor ((Element or CSSPseudoElement)? target,
              object? keyframes,
              optional (unrestricted double or KeyframeEffectOptions) options),
 Constructor (KeyframeEffect source)]
interface KeyframeEffect : AnimationEffect {
  attribute (Element or CSSPseudoElement)?  target;
  [NeedsCallerType]
  attribute IterationCompositeOperation     iterationComposite;
  attribute CompositeOperation              composite;
  [Throws] sequence<object> getKeyframes ();
  [Throws] undefined             setKeyframes (object? keyframes);
};

// Non-standard extensions
dictionary AnimationPropertyValueDetails {
  required double             offset;
           DOMString          value;
           DOMString          easing;
  required CompositeOperation composite;
};

dictionary AnimationPropertyDetails {
  required DOMString                               property;
  required boolean                                 runningOnCompositor;
           DOMString                               warning;
  required sequence<AnimationPropertyValueDetails> values;
};

partial interface KeyframeEffect {
  [ChromeOnly, Throws] sequence<AnimationPropertyDetails> getProperties();
};
