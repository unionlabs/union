/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Various functions useful for automated fuzzing that are enabled
 * only in --enable-fuzzing builds, because they may be dangerous to
 * enable on untrusted pages.
*/

[Pref="fuzzing.enabled"]
interface FuzzingFunctions {
  /**
   * Synchronously perform a garbage collection.
   */
  static undefined garbageCollect();

  /**
   * Synchronously perform a cycle collection.
   */
  static undefined cycleCollect();

  /**
   * Enable accessibility.
   */
  [Throws]
  static undefined enableAccessibility();
};
