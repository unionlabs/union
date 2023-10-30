/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * Source: https://streams.spec.whatwg.org/#transformstream
 */
 
[Exposed=(Window,Worker,Worklet), Transferable]
interface TransformStream {
  constructor(optional object transformer, optional QueuingStrategy writableStrategy = {}, optional QueuingStrategy readableStrategy = {});

  readonly attribute ReadableStream readable;
  readonly attribute WritableStream writable;
};
