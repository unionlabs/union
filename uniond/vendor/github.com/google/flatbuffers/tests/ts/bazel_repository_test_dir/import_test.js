import assert from 'assert'
import * as flatbuffers from 'flatbuffers'

import two_cjs from './two_generated.cjs'

const bazel_repository_test = two_cjs.bazel_repository_test;

function main() {
  // Validate building a table with a table field.
  var fbb = new flatbuffers.Builder(1);

  bazel_repository_test.One.startOne(fbb);
  bazel_repository_test.One.addInformation(fbb, 42);
  var one = bazel_repository_test.One.endOne(fbb);

  bazel_repository_test.Two.startTwo(fbb);
  bazel_repository_test.Two.addOne(fbb, one);
  var two = bazel_repository_test.Two.endTwo(fbb);

  fbb.finish(two);

  // Call as a sanity check. Would be better to validate actual output here.
  fbb.asUint8Array();

  console.log('FlatBuffers bazel repository test: completed successfully');
}

main();
