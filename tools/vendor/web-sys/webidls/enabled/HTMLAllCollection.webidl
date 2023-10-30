/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/* Emulates undefined through Codegen.py. */
[LegacyUnenumerableNamedProperties]
interface HTMLAllCollection {
  readonly attribute unsigned long length;
  getter Node? (unsigned long index);
  Node? item(unsigned long index);
  (Node or HTMLCollection)? item(DOMString name);
  legacycaller (Node or HTMLCollection)? (DOMString name);
  getter (Node or HTMLCollection)? namedItem(DOMString name);
};
