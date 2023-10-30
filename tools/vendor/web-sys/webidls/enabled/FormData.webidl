/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://xhr.spec.whatwg.org
 */

typedef (Blob or Directory or USVString) FormDataEntryValue;

[Constructor(optional HTMLFormElement form),
 Exposed=(Window,Worker)]
interface FormData {
  [Throws]
  undefined append(USVString name, Blob value, optional USVString filename);
  [Throws]
  undefined append(USVString name, USVString value);
  undefined delete(USVString name);
  FormDataEntryValue? get(USVString name);
  sequence<FormDataEntryValue> getAll(USVString name);
  boolean has(USVString name);
  [Throws]
  undefined set(USVString name, Blob value, optional USVString filename);
  [Throws]
  undefined set(USVString name, USVString value);
  iterable<USVString, FormDataEntryValue>;
};
