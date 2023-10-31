/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://storage.spec.whatwg.org/#storagemanager
 *
 */

[SecureContext,
 Exposed=(Window,Worker),
 Func="mozilla::dom::DOMPrefs::StorageManagerEnabled"]
interface StorageManager {
  [Throws]
  Promise<boolean> persisted();

  [Exposed=Window, Throws]
  Promise<boolean> persist();

  [Throws]
  Promise<StorageEstimate> estimate();
};

dictionary StorageEstimate {
  unsigned long long usage;
  unsigned long long quota;
};
