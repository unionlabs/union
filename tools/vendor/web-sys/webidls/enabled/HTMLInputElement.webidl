/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://www.whatwg.org/specs/web-apps/current-work/#the-input-element
 * http://www.whatwg.org/specs/web-apps/current-work/#other-elements,-attributes-and-apis
 *
 * © Copyright 2004-2011 Apple Computer, Inc., Mozilla Foundation, and
 * Opera Software ASA. You are granted a license to use, reproduce
 * and create derivative works of this document.
 */

/*TODO
enum SelectionMode {
  "select",
  "start",
  "end",
  "preserve",
};

interface XULControllers;
*/

[HTMLConstructor]
interface HTMLInputElement : HTMLElement {
  [CEReactions, Pure, SetterThrows]
           attribute DOMString accept;
  [CEReactions, Pure, SetterThrows]
           attribute DOMString alt;
  [CEReactions, Pure, SetterThrows]
           attribute DOMString autocomplete;
  [CEReactions, Pure, SetterThrows]
           attribute boolean autofocus;
  [CEReactions, Pure, SetterThrows]
           attribute boolean defaultChecked;
  [Pure]
           attribute boolean checked;
           // Bug 850337 - attribute DOMString dirName;
  [CEReactions, Pure, SetterThrows]
           attribute boolean disabled;
  readonly attribute HTMLFormElement? form;
  [Pure]
           attribute FileList? files;
  [CEReactions, Pure, SetterThrows]
           attribute DOMString formAction;
  [CEReactions, Pure, SetterThrows]
           attribute DOMString formEnctype;
  [CEReactions, Pure, SetterThrows]
           attribute DOMString formMethod;
  [CEReactions, Pure, SetterThrows]
           attribute boolean formNoValidate;
  [CEReactions, Pure, SetterThrows]
           attribute DOMString formTarget;
  [CEReactions, Pure, SetterThrows]
           attribute unsigned long height;
  [Pure]
           attribute boolean indeterminate;
  [CEReactions, Pure, SetterThrows, Pref="dom.forms.inputmode"]
           attribute DOMString inputMode;
  [Pure]
  readonly attribute HTMLElement? list;
  [CEReactions, Pure, SetterThrows]
           attribute DOMString max;
  [CEReactions, Pure, SetterThrows]
           attribute long maxLength;
  [CEReactions, Pure, SetterThrows]
           attribute DOMString min;
  [CEReactions, Pure, SetterThrows]
           attribute long minLength;
  [CEReactions, Pure, SetterThrows]
           attribute boolean multiple;
  [CEReactions, Pure, SetterThrows]
           attribute DOMString name;
  [CEReactions, Pure, SetterThrows]
           attribute DOMString pattern;
  [CEReactions, Pure, SetterThrows]
           attribute DOMString placeholder;
  [CEReactions, Pure, SetterThrows]
           attribute boolean readOnly;
  [CEReactions, Pure, SetterThrows]
           attribute boolean required;
  [CEReactions, Pure, SetterThrows]
           attribute unsigned long size;
  [CEReactions, Pure, SetterNeedsSubjectPrincipal=NonSystem, SetterThrows]
           attribute DOMString src;
  [CEReactions, Pure, SetterThrows]
           attribute DOMString step;
  [CEReactions, Pure, SetterThrows]
           attribute DOMString type;
  [CEReactions, Pure, SetterThrows]
           attribute DOMString defaultValue;
  [CEReactions, Pure, TreatNullAs=EmptyString, SetterThrows, NeedsCallerType]
           attribute DOMString value;
  [Throws, Func="HTMLInputElement::ValueAsDateEnabled"]
           attribute Date? valueAsDate;
  [Pure, SetterThrows]
           attribute unrestricted double valueAsNumber;
  [CEReactions, SetterThrows]
           attribute unsigned long width;
/* TODO
  [Throws]
  undefined stepUp(optional long n = 1);
  [Throws]
  undefined stepDown(optional long n = 1);
*/

  [Pure]
  readonly attribute boolean willValidate;
  [Pure]
  readonly attribute ValidityState validity;
  [Throws]
  readonly attribute DOMString validationMessage;
  boolean checkValidity();
  boolean reportValidity();
  undefined setCustomValidity(DOMString error);

  readonly attribute NodeList? labels;

  undefined select();

  [Throws]
           attribute unsigned long? selectionStart;
  [Throws]
           attribute unsigned long? selectionEnd;
  [Throws]
           attribute DOMString? selectionDirection;
  [Throws]
  undefined setRangeText(DOMString replacement);

  [Throws]
  undefined setRangeText(DOMString replacement, unsigned long start,
    unsigned long end, optional SelectionMode selectionMode = "preserve");
  [Throws]
  undefined setSelectionRange(unsigned long start, unsigned long end, optional DOMString direction);

  // also has obsolete members
};

partial interface HTMLInputElement {
  [CEReactions, Pure, SetterThrows]
           attribute DOMString align;
  [CEReactions, Pure, SetterThrows]
           attribute DOMString useMap;
};

/*Non standard
partial interface HTMLInputElement {
  [Pref="dom.input.dirpicker", SetterThrows]
  attribute boolean allowdirs;

  [Pref="dom.input.dirpicker"]
  readonly attribute boolean isFilesAndDirectoriesSupported;

  [Throws, Pref="dom.input.dirpicker"]
  Promise<sequence<(File or Directory)>> getFilesAndDirectories();

  [Throws, Pref="dom.input.dirpicker"]
  Promise<sequence<File>> getFiles(optional boolean recursiveFlag = false);

  [Throws, Pref="dom.input.dirpicker"]
  undefined chooseDirectory();
};
*/

// Webkit/Blink
partial interface HTMLInputElement {
  [Pref="dom.webkitBlink.filesystem.enabled", Frozen, Cached, Pure]
  readonly attribute sequence<FileSystemEntry> webkitEntries;

  [Pref="dom.webkitBlink.dirPicker.enabled", BinaryName="WebkitDirectoryAttr", SetterThrows]
          attribute boolean webkitdirectory;
};

dictionary DateTimeValue {
  long hour;
  long minute;
  long year;
  long month;
  long day;
};

partial interface HTMLInputElement {
  [Pref="dom.forms.datetime", ChromeOnly]
  DateTimeValue getDateTimeInputBoxValue();

  [Pref="dom.forms.datetime", ChromeOnly]
  undefined updateDateTimeInputBox(optional DateTimeValue value);

  [Pref="dom.forms.datetime", ChromeOnly]
  undefined setDateTimePickerState(boolean open);

  [Pref="dom.forms.datetime", ChromeOnly,
   BinaryName="getMinimumAsDouble"]
  double getMinimum();

  [Pref="dom.forms.datetime", ChromeOnly,
   BinaryName="getMaximumAsDouble"]
  double getMaximum();
};

partial interface HTMLInputElement {
  [ChromeOnly]
  attribute DOMString previewValue;
};
