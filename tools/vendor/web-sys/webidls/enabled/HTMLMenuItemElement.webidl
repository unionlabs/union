/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://www.whatwg.org/specs/web-apps/current-work/#the-menuitem-element
 *
 * © Copyright 2004-2011 Apple Computer, Inc., Mozilla Foundation, and
 * Opera Software ASA. You are granted a license to use, reproduce
 * and create derivative works of this document.
 */

// http://www.whatwg.org/specs/web-apps/current-work/#the-menuitem-element
[HTMLConstructor]
interface HTMLMenuItemElement : HTMLElement {
           [CEReactions, SetterThrows]
           attribute DOMString type;
           [CEReactions, SetterThrows]
           attribute DOMString label;
           [CEReactions, SetterThrows]
           attribute DOMString icon;
           [CEReactions, SetterThrows]
           attribute boolean disabled;
           [CEReactions]
           attribute boolean checked;
           [CEReactions, SetterThrows]
           attribute DOMString radiogroup;

           // This should be 'default' but in the IDL implementation
           // this has been renamed 'defaultChecked'.
           [CEReactions, SetterThrows]
           attribute boolean defaultChecked;

           // Currently not implemented.
//  readonly attribute HTMLElement? command;
};
