/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/**
 * The following dictionaries are for Mozilla use only. They allow startup
 * localization runtime to work around the performance cost of Stylo having
 * to resolve XBL bindings in order to localize DOM in JS.
 *
 * Instead, we use `Node.localize` method which handles scanning for localizable
 * elements and applies the result translations without having to create
 * JS reflections for them.
 *
 * For details on the implementation of the API, see `Node.webidl`.
 */
dictionary L10nElement {
  required DOMString namespaceURI;
  required DOMString localName;
  required DOMString l10nId; // value of data-l10n-id
  DOMString? type = null;
  DOMString? l10nAttrs = null; // value of data-l10n-attrs
  object? l10nArgs = null; // json value of data-l10n-args attribute
};

dictionary AttributeNameValue {
  required DOMString name;
  required DOMString value;
};

dictionary L10nValue {
  DOMString? value = null;
  sequence<AttributeNameValue>? attributes = null;
};

callback L10nCallback =
  Promise<sequence<L10nValue>> (sequence<L10nElement> l10nElements);
