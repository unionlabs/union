/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/. */

dictionary DisplayNameOptions {
  DOMString style;
  sequence<DOMString> keys;
};

dictionary DisplayNameResult {
  DOMString locale;
  DOMString style;
  record<DOMString, DOMString> values;
};

dictionary LocaleInfo {
  DOMString locale;
  DOMString direction;
};

/**
 * The IntlUtils interface provides helper functions for localization.
 */
[NoInterfaceObject]
interface IntlUtils {
  /**
   * Helper function to retrieve the localized values for a list of requested
   * keys.
   *
   * The function takes two arguments - locales which is a list of locale
   * strings and options which is an object with two optional properties:
   *
   *   keys:
   *     an Array of string values that are paths to individual terms
   *
   *   style:
   *     a String with a value "long", "short" or "narrow"
   *
   * It returns an object with properties:
   *
   *   locale:
   *     a negotiated locale string
   *
   *   style:
   *     negotiated style
   *
   *   values:
   *     a key-value pair list of requested keys and corresponding translated
   *     values
   *
   */
  [Throws]
  DisplayNameResult getDisplayNames(sequence<DOMString> locales,
                                    optional DisplayNameOptions options);

  /**
   * Helper function to retrieve useful information about a locale.
   *
   * The function takes one argument - locales which is a list of locale
   * strings.
   *
   * It returns an object with properties:
   *
   *   locale:
   *     a negotiated locale string
   *
   *   direction:
   *     text direction, "ltr" or "rtl"
   *
   */
  [Throws]
  LocaleInfo getLocaleInfo(sequence<DOMString> locales);
};
