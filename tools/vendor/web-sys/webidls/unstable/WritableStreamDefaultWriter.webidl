/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * Source: https://streams.spec.whatwg.org/#writablestreamdefaultwriter
 */
 
[Exposed=(Window,Worker,Worklet)]
interface WritableStreamDefaultWriter {
  constructor(WritableStream stream);

  readonly attribute Promise<undefined> closed;
  readonly attribute unrestricted double? desiredSize;
  readonly attribute Promise<undefined> ready;

  Promise<undefined> abort(optional any reason);
  Promise<undefined> close();
  undefined releaseLock();
  Promise<undefined> write(optional any chunk);
};
