'use strict';

var SwaggerClient = require('swagger-client');

describe('ABitOfEverythingService', function () {
  var client;

  beforeEach(function (done) {
    new SwaggerClient({
      url: "http://localhost:8080/openapiv2/a_bit_of_everything.swagger.json",
      usePromise: true,
    }).then(function (c) {
      client = c;
    }).catch(function (err) {
      done.fail(err);
    }).then(done);
  });

  describe('Create', function () {
    var created;
    var expected = {
      floatValue: 1.5,
      doubleValue: 2.5,
      int64Value: "4294967296",
      uint64Value: "9223372036854775807",
      int32Value: -2147483648,
      fixed64Value: "9223372036854775807",
      fixed32Value: 4294967295,
      boolValue: true,
      stringValue: "strprefix/foo",
      uint32Value: 4294967295,
      sfixed32Value: 2147483647,
      sfixed64Value: "-4611686018427387904",
      sint32Value: 2147483647,
      sint64Value: "4611686018427387903",
      nonConventionalNameValue: "camelCase",
      enumValue: "ONE",
      pathEnumValue: "DEF",
      nestedPathEnumValue: "JKL",
      enumValueAnnotation: "ONE",
      requiredStringViaFieldBehaviorAnnotation: "foo",
      singleNested: null,
      nested: [],
      bytesValue: "",
      repeatedStringValue: [],
      mapValue: {},
      mappedStringValue: {},
      mappedNestedValue: {},
      timestampValue: "2006-01-02T15:04:05Z",
      repeatedEnumValue: [],
      repeatedEnumAnnotation: [],
      repeatedStringAnnotation: [],
      repeatedNestedAnnotation: [],
      nestedAnnotation: null,
      int64OverrideType: "0",
      outputOnlyStringViaFieldBehaviorAnnotation: "",
    };

    beforeEach(function (done) {
      client.ABitOfEverythingService.ABitOfEverythingService_Create(expected).then(function (resp) {
        created = resp.obj;
      }).catch(function (err) {
        done.fail(err);
      }).then(done);
    });

    it('should assign id', function () {
      expect(created.uuid).not.toBe("");
    });

    it('should echo the request back', function () {
      delete created.uuid;
      expect(created).toEqual(expected);
    });
  });

  describe('CreateBody', function () {
    var created;
    var expected = {
      floatValue: 1.5,
      doubleValue: 2.5,
      int64Value: "4294967296",
      uint64Value: "9223372036854775807",
      int32Value: -2147483648,
      fixed64Value: "9223372036854775807",
      fixed32Value: 4294967295,
      boolValue: true,
      stringValue: "strprefix/foo",
      uint32Value: 4294967295,
      sfixed32Value: 2147483647,
      sfixed64Value: "-4611686018427387904",
      sint32Value: 2147483647,
      sint64Value: "4611686018427387903",
      nonConventionalNameValue: "camelCase",
      enumValue: "ONE",
      pathEnumValue: "DEF",
      nestedPathEnumValue: "JKL",
      nested: [
        { name: "bar", amount: 10 },
        { name: "baz", amount: 20 },
      ],
      repeatedStringValue: ["a", "b", "c"],
      oneofString: "x",
      mapValue: { a: "ONE", b: 2 },
      mappedStringValue: { a: "x", b: "y" },
      mappedNestedValue: {
        a: { name: "x", amount: 1 },
        b: { name: "y", amount: 2 },
      },
      enumValueAnnotation: "ONE",
      requiredStringViaFieldBehaviorAnnotation: "foo",
      singleNested: null,
      nested: [],
      bytesValue: "",
      repeatedStringValue: [],
      mapValue: {},
      mappedStringValue: {},
      mappedNestedValue: {},
      timestampValue: "2006-01-02T15:04:05Z",
      repeatedEnumValue: [],
      repeatedEnumAnnotation: [],
      repeatedStringAnnotation: [],
      repeatedNestedAnnotation: [],
      nestedAnnotation: null,
      int64OverrideType: "0",
      outputOnlyStringViaFieldBehaviorAnnotation: "",
    };

    beforeEach(function (done) {
      client.ABitOfEverythingService.ABitOfEverythingService_CreateBody({
        body: expected,
      }).then(function (resp) {
        created = resp.obj;
      }).catch(function (err) {
        done.fail(err);
      }).then(done);
    });

    it('should assign id', function () {
      expect(created.uuid).not.toBe("");
    });

    it('should echo the request back', function () {
      delete created.uuid;
      expect(created).toEqual(expected);
    });
  });

  describe('lookup', function () {
    var created;
    var expected = {
      boolValue: true,
      stringValue: "strprefix/foo",
    };

    beforeEach(function (done) {
      client.ABitOfEverythingService.ABitOfEverythingService_CreateBody({
        body: expected,
      }).then(function (resp) {
        created = resp.obj;
      }).catch(function (err) {
        fail(err);
      }).finally(done);
    });

    it('should look up an object by uuid', function (done) {
      client.ABitOfEverythingService.ABitOfEverythingService_Lookup({
        uuid: created.uuid
      }).then(function (resp) {
        expect(resp.obj).toEqual(created);
      }).catch(function (err) {
        fail(err.errObj);
      }).finally(done);
    });

    it('should fail if no such object', function (done) {
      client.ABitOfEverythingService.ABitOfEverythingService_Lookup({
        uuid: 'not_exist',
      }).then(function (resp) {
        fail('expected failure but succeeded');
      }).catch(function (err) {
        expect(err.status).toBe(404);
      }).finally(done);
    });
  });

  describe('Delete', function () {
    var created;
    var expected = {
      boolValue: true,
      stringValue: "strprefix/foo",
    };

    beforeEach(function (done) {
      client.ABitOfEverythingService.ABitOfEverythingService_CreateBody({
        body: expected,
      }).then(function (resp) {
        created = resp.obj;
      }).catch(function (err) {
        fail(err);
      }).finally(done);
    });

    it('should delete an object by id', function (done) {
      client.ABitOfEverythingService.ABitOfEverythingService_Delete({
        uuid: created.uuid
      }).then(function (resp) {
        expect(resp.obj).toEqual({});
      }).catch(function (err) {
        fail(err.errObj);
      }).then(function () {
        return client.ABitOfEverythingService.ABitOfEverythingService_Lookup({
          uuid: created.uuid
        });
      }).then(function (resp) {
        fail('expected failure but succeeded');
      }).catch(function (err) {
        expect(err.status).toBe(404);
      }).finally(done);
    });
  });

  describe('GetRepeatedQuery', function () {
    var repeated;
    var expected = {
      pathRepeatedFloatValue: [1.5, -1.5],
      pathRepeatedDoubleValue: [2.5, -2.5],
      pathRepeatedInt64Value: ["4294967296", "-4294967296"],
      pathRepeatedUint64Value: ["0", "9223372036854775807"],
      pathRepeatedInt32Value: [2147483647, -2147483648],
      pathRepeatedFixed64Value: ["0", "9223372036854775807"],
      pathRepeatedFixed32Value: [0, 4294967295],
      pathRepeatedBoolValue: [true, false],
      pathRepeatedStringValue: ["foo", "bar"],
      pathRepeatedBytesValue: ["AA==", "_w=="],
      pathRepeatedUint32Value: [4294967295, 0],
      pathRepeatedEnumValue: ["ONE", "ONE"],
      pathRepeatedSfixed32Value: [-2147483648, 2147483647],
      pathRepeatedSfixed64Value: ["-4294967296", "4294967296"],
      pathRepeatedSint32Value: [2147483646, -2147483647],
      pathRepeatedSint64Value: ["4611686018427387903", "-4611686018427387904"]
    };

    beforeEach(function (done) {
      client.ABitOfEverythingService.ABitOfEverythingService_GetRepeatedQuery(expected).then(function (resp) {
        repeated = resp.obj;
      }).catch(function (err) {
        done.fail(err);
      }).then(done);
    });

    it('should echo the request back', function () {
      // API will echo a non URL safe encoding
      expected.pathRepeatedBytesValue = ["AA==", "/w=="];
      expect(repeated).toEqual(expected);
    });
  });
});
