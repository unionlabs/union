/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * For more information on this interface, please see
 * http://slightlyoff.github.io/ServiceWorker/spec/service_worker/index.html
 */

[Constructor(DOMString type, optional ExtendableEventInit eventInitDict),
 Exposed=ServiceWorker]
interface ExtendableEvent : Event {
  // https://github.com/slightlyoff/ServiceWorker/issues/261
  [Throws]
  undefined waitUntil(Promise<any> p);
};

dictionary ExtendableEventInit : EventInit {
  // Defined for the forward compatibility across the derived events
};
