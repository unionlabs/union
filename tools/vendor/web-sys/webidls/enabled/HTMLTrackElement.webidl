/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://www.whatwg.org/specs/web-apps/current-work/#the-track-element
 */

[HTMLConstructor]
interface HTMLTrackElement : HTMLElement {
  [CEReactions, SetterThrows, Pure]
  attribute DOMString kind;
  [CEReactions, SetterThrows, Pure]
  attribute DOMString src;
  [CEReactions, SetterThrows, Pure]
  attribute DOMString srclang;
  [CEReactions, SetterThrows, Pure]
  attribute DOMString label;
  [CEReactions, SetterThrows, Pure]
  attribute boolean default;

  const unsigned short NONE = 0;
  const unsigned short LOADING = 1;
  const unsigned short LOADED = 2;
  const unsigned short ERROR = 3;
  readonly attribute unsigned short readyState;

  readonly attribute TextTrack? track;
};
