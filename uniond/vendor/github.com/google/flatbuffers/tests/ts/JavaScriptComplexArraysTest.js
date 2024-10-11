/* global BigInt */

import assert from 'assert';
import { readFileSync, writeFileSync } from 'fs';
import * as flatbuffers from 'flatbuffers';
import { ArrayStructT } from './arrays_test_complex/my-game/example/array-struct.js'
import { ArrayTable, ArrayTableT } from './arrays_test_complex/my-game/example/array-table.js'
import { InnerStructT } from './arrays_test_complex/my-game/example/inner-struct.js'
import { NestedStructT } from './arrays_test_complex/my-game/example/nested-struct.js'
import { OuterStructT } from './arrays_test_complex/my-game/example/outer-struct.js'
import { TestEnum } from './arrays_test_complex/my-game/example/test-enum.js'
// eslint-disable-next-line @typescript-eslint/no-explicit-any
BigInt.prototype.toJSON = function () {
  return this.toString();
};
function fbObjToObj(fbObj) {
  const ret = {};
  for (const propName of Object.keys(fbObj)) {
    const key = propName;
    const prop = fbObj[key];
    if (prop.valueOf) {
      ret[key] = prop.valueOf();
    } else if (typeof prop === 'object') {
      ret[key] = fbObjToObj(prop);
    }
  }
  return ret;
}
function testBuild(monFile, jsFile) {
  const arrayTable = new ArrayTableT(
    'Complex Array Test',
    new ArrayStructT(
      221.139008,
      [-700, -600, -500, -400, -300, -200, -100, 0, 100, 200, 300, 400, 500, 600, 700],
      13,
      [
        new NestedStructT(
          [233, -123],
          TestEnum.B,
          [TestEnum.A, TestEnum.C],
          [
            new OuterStructT(
              false,
              123.456,
              new InnerStructT(
                123456792.0,
                [13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1],
                91,
                BigInt('9007199254740999')
              ),
              [
                new InnerStructT(
                  -987654321.9876,
                  [255, 254, 253, 252, 251, 250, 249, 248, 247, 246, 245, 244, 243],
                  123,
                  BigInt('9007199254741000')
                ),
                new InnerStructT(
                  123000987.9876,
                  [101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113],
                  -123,
                  BigInt('9007199254741000')
                ),
              ],
              new InnerStructT(
                987654321.9876,
                [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13],
                19,
                BigInt('9007199254741000')
              ),
              [111000111.222, 222000222.111, 333000333.333, 444000444.444]
            ),
          ]
        ),
      ],
      -123456789
    )
  );
  const builder = new flatbuffers.Builder();
  builder.finish(arrayTable.pack(builder));
  if (jsFile) {
    const obj = fbObjToObj(arrayTable);
    writeFileSync(jsFile, `export default ${JSON.stringify(obj, null, 2)}`);
  }
  if (monFile) {
    writeFileSync(monFile, builder.asUint8Array());
  }
  return builder.asUint8Array();
}
function testParse(monFile, jsFile, buffer) {
  if (!buffer) {
    if (!monFile) {
      console.log(`Please specify mon file read the buffer from.`);
      process.exit(1);
    }
    buffer = readFileSync(monFile);
  }
  const byteBuffer = new flatbuffers.ByteBuffer(new Uint8Array(buffer));
  const arrayTable = ArrayTable.getRootAsArrayTable(byteBuffer).unpack();
  const json = JSON.stringify(arrayTable, null, 2);
  if (jsFile) {
    writeFileSync(jsFile, `export default ${json}`);
  }
  return arrayTable;
}
if (process.argv[2] === 'build') {
  testBuild(process.argv[3], process.argv[4]);
} else if (process.argv[2] === 'parse') {
  testParse(process.argv[3], process.argv[4], null);
} else {
  const arr = testBuild(null, null);
  const parsed = testParse(null, null, Buffer.from(arr));
  assert.strictEqual(parsed.a, 'Complex Array Test', 'String Test');
  assert.strictEqual(parsed?.cUnderscore?.aUnderscore, 221.13900756835938, 'Float Test');
  assert.deepEqual(parsed?.cUnderscore?.bUnderscore, [-700, -600, -500, -400, -300, -200, -100, 0, 100, 200, 300, 400, 500, 600, 700], 'Array of signed integers');
  assert.strictEqual(parsed?.cUnderscore.d?.[0].dOuter[0].d[1].a, 123000987.9876, 'Float in deep');
  assert.deepEqual(parsed?.cUnderscore?.d[0].dOuter?.[0]?.e, {
    a: 987654321.9876,
    b: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13],
    c: 19,
    dUnderscore: '9007199254741000',
  }, 'Object in deep');
  assert.deepEqual(parsed?.cUnderscore.g, ['0', '0'], 'Last object');

  console.log('Arrays test: completed successfully');
}
