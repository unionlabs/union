/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * Source: https://streams.spec.whatwg.org/#readablestreamgenericreader
 */
 
interface mixin ReadableStreamGenericReader {
  readonly attribute Promise<undefined> closed;

  Promise<undefined> cancel(optional any reason);
};
