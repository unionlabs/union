/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * For more information on this interface, please see
 * https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#submitevent
 */

[Constructor(DOMString type, optional SubmitEventInit eventInitDict),
    Exposed=(Window)]
interface SubmitEvent : Event {
  readonly attribute HTMLElement? submitter;
};

dictionary SubmitEventInit : EventInit {
  HTMLElement? submitter = null;
};
