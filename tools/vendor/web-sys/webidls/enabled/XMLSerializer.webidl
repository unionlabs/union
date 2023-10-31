/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://domparsing.spec.whatwg.org/#the-xmlserializer-interface
 */

// invalid widl
// interface OutputStream;

[Constructor]
interface XMLSerializer {
  /**
   * The subtree rooted by the specified element is serialized to
   * a string.
   *
   * @param root The root of the subtree to be serialized. This could
   *             be any node, including a Document.
   * @returns The serialized subtree in the form of a Unicode string
   */
  [Throws]
  DOMString serializeToString(Node root);
};
