/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/**
 * This dictionary is used for the input, textarea and select element's
 * getAutocompleteInfo method.
 */

dictionary AutocompleteInfo {
  DOMString section = "";
  DOMString addressType = "";
  DOMString contactType = "";
  DOMString fieldName = "";
};
