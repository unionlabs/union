/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/**
 * A fake plugin is fundamentally identified by its handlerURI.
 *
 * In addition to that, a fake plugin registration needs to provide at least one
 * FakePluginMimeEntry so we'll know what types(s) the plugin is registered for.
 * Other information is optional, though having usable niceName is highly
 * recommended.
 */
dictionary FakePluginTagInit {
  required DOMString handlerURI;
  required sequence<FakePluginMimeEntry> mimeEntries;

  // The niceName should really be provided, and be unique, if possible; it can
  // be used as a key to persist state for this plug-in.
  DOMString niceName = "";

  // Other things can be provided but don't really matter that much.
  DOMString fullPath = "";
  DOMString name = "";
  DOMString description = "";
  DOMString fileName = "";
  DOMString version = "";

  /**
   * Optional script to run in a sandbox when instantiating a plugin. The script
   * runs in a sandbox with system principal in the process that contains the
   * element that instantiates the plugin (ie the EMBED or OBJECT element). The
   * sandbox global has a 'pluginElement' property that the script can use to
   * access the element that instantiates the plugin.
   */
  DOMString sandboxScript = "";
};

/**
 * A single MIME entry for the fake plugin.
 */
dictionary FakePluginMimeEntry {
  required DOMString type;
  DOMString description = "";
  DOMString extension = "";
};

