/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://www.whatwg.org/html/#the-select-element
 */

[HTMLConstructor]
interface HTMLSelectElement : HTMLElement {
  [CEReactions, SetterThrows, Pure]
  attribute boolean autofocus;
  [CEReactions, SetterThrows, Pure]
  attribute DOMString autocomplete;
  [CEReactions, SetterThrows, Pure]
  attribute boolean disabled;
  [Pure]
  readonly attribute HTMLFormElement? form;
  [CEReactions, SetterThrows, Pure]
  attribute boolean multiple;
  [CEReactions, SetterThrows, Pure]
  attribute DOMString name;
  [CEReactions, SetterThrows, Pure]
  attribute boolean required;
  [CEReactions, SetterThrows, Pure]
  attribute unsigned long size;

  [Pure]
  readonly attribute DOMString type;

  [Constant]
  readonly attribute HTMLOptionsCollection options;
  [CEReactions, SetterThrows, Pure]
  attribute unsigned long length;
  getter Element? item(unsigned long index);
  HTMLOptionElement? namedItem(DOMString name);
  [CEReactions, Throws]
  undefined add((HTMLOptionElement or HTMLOptGroupElement) element, optional (HTMLElement or long)? before = null);
  [CEReactions]
  undefined remove(long index);
  [CEReactions, Throws]
  setter undefined (unsigned long index, HTMLOptionElement? option);

  readonly attribute HTMLCollection selectedOptions;
  [SetterThrows, Pure]
  attribute long selectedIndex;
  [Pure]
  attribute DOMString value;

  readonly attribute boolean willValidate;
  readonly attribute ValidityState validity;
  [Throws]
  readonly attribute DOMString validationMessage;
  boolean checkValidity();
  boolean reportValidity();
  undefined setCustomValidity(DOMString error);

  readonly attribute NodeList labels;

  // https://www.w3.org/Bugs/Public/show_bug.cgi?id=20720
  [CEReactions]
  undefined remove();
};

// Chrome only interface
partial interface HTMLSelectElement {
  [ChromeOnly]
  attribute boolean openInParentProcess;
  [ChromeOnly]
  AutocompleteInfo getAutocompleteInfo();
  [ChromeOnly]
  attribute DOMString previewValue;
};
