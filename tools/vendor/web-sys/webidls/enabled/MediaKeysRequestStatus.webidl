/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 */

enum MediaKeySystemStatus {
  "available",
  "api-disabled",
  "cdm-disabled",
  "cdm-not-supported",
  "cdm-not-installed",
  "cdm-created",
};

/* Note: This dictionary and enum is only used by Gecko to convey messages
 * to chrome JS code. It is not exposed to the web.
 */

dictionary RequestMediaKeySystemAccessNotification {
  required DOMString keySystem;
  required MediaKeySystemStatus status;
};
