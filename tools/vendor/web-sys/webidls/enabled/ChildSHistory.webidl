/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 */

// invalid widl
///interface nsISHistory;

/**
 * The ChildSHistory interface represents the child side of a browsing
 * context's session history.
 */
[ChromeOnly]
interface ChildSHistory {
  [Pure]
  readonly attribute long count;
  [Pure]
  readonly attribute long index;

  boolean canGo(long aOffset);
  [Throws]
  undefined go(long aOffset);

  /**
   * Reload the current entry. The flags which should be passed to this
   * function are documented and defined in nsIWebNavigation.idl
   */
  [Throws]
  undefined reload(unsigned long aReloadFlags);

  /**
   * Getter for the legacy nsISHistory implementation.
   *
   * This getter _will be going away_, but is needed while we finish
   * implementing all of the APIs which we will need in the content
   * process on ChildSHistory.
   */
  readonly attribute nsISHistory legacySHistory;
};
