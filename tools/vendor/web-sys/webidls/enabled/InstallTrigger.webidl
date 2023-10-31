/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 */


/**
 * A callback function that webpages can implement to be notified when triggered
 * installs complete.
 */
callback InstallTriggerCallback = undefined(DOMString url, short status);

dictionary InstallTriggerData {
  DOMString URL;
  DOMString? IconURL;
  DOMString? Hash;
};

/**
 * The interface for the InstallTrigger object available to all websites.
 */
[ChromeOnly,
 JSImplementation="@mozilla.org/addons/installtrigger;1"]
interface InstallTriggerImpl {
  /**
   * Retained for backwards compatibility.
   */
  const unsigned short SKIN = 1;
  const unsigned short LOCALE = 2;
  const unsigned short CONTENT = 4;
  const unsigned short PACKAGE = 7;

  /**
   * Tests if installation is enabled.
   */
  boolean enabled();

  /**
   * Tests if installation is enabled.
   *
   * @deprecated Use "enabled" in the future.
   */
  boolean updateEnabled();

  /**
   * Starts a new installation of a set of add-ons.
   *
   * @param  aArgs
   *         The add-ons to install. This should be a JS object, each property
   *         is the name of an add-on to be installed. The value of the
   *         property should either be a string URL, or an object with the
   *         following properties:
   *          * URL for the add-on's URL
   *          * IconURL for an icon for the add-on
   *          * Hash for a hash of the add-on
   * @param  aCallback
   *         A callback to call as each installation succeeds or fails
   * @return true if the installations were successfully started
   */
  boolean install(record<DOMString, (DOMString or InstallTriggerData)> installs,
                  optional InstallTriggerCallback callback);

  /**
   * Starts installing a new add-on.
   *
   * @deprecated use "install" in the future.
   *
   * @param  aType
   *         Unused, retained for backwards compatibility
   * @param  aUrl
   *         The URL of the add-on
   * @param  aSkin
   *         Unused, retained for backwards compatibility
   * @return true if the installation was successfully started
   */
  boolean installChrome(unsigned short type, DOMString url, DOMString skin);

  /**
   * Starts installing a new add-on.
   *
   * @deprecated use "install" in the future.
   *
   * @param  aUrl
   *         The URL of the add-on
   * @param  aFlags
   *         Unused, retained for backwards compatibility
   * @return true if the installation was successfully started
   */
  boolean startSoftwareUpdate(DOMString url, optional unsigned short flags);
};
