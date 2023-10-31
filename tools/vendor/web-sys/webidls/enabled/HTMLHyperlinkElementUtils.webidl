/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://html.spec.whatwg.org/multipage/semantics.html#htmlhyperlinkelementutils
 * © Copyright 2004-2011 Apple Computer, Inc., Mozilla Foundation, and
 * Opera Software ASA. You are granted a license to use, reproduce
 * and create derivative works of this document.
 */

interface mixin HTMLHyperlinkElementUtils {
  // Bug 824857: no support for stringifier attributes yet.
  //  stringifier attribute USVString href;

  // Bug 824857 should remove this.
  stringifier;

  [CEReactions, SetterThrows]
           attribute USVString href;

  readonly attribute USVString origin;
  [CEReactions]
           attribute USVString protocol;
  [CEReactions]
           attribute USVString username;
  [CEReactions]
           attribute USVString password;
  [CEReactions]
           attribute USVString host;
  [CEReactions]
           attribute USVString hostname;
  [CEReactions]
           attribute USVString port;
  [CEReactions]
           attribute USVString pathname;
  [CEReactions]
           attribute USVString search;
  [CEReactions]
           attribute USVString hash;
};
