'use strict';

var SwaggerClient = require('swagger-client');

describe('EchoService', function () {
  var client;

  beforeEach(function (done) {
    new SwaggerClient({
      url: "http://localhost:8080/openapiv2/echo_service.swagger.json",
      usePromise: true,
    }).then(function (c) {
      client = c;
      done();
    });
  });

  describe('Echo', function () {
    it('should echo the request back', function (done) {
      var expected = {
        id: "foo",
        num: "0",
        status: null
      };
      client.EchoService.EchoService_Echo(
        expected,
        { responseContentType: "application/json" }
      ).then(function (resp) {
        expect(resp.obj).toEqual(expected);
      }).catch(function (err) {
        done.fail(err);
      }).then(done);
    });
  });

  describe('EchoBody', function () {
    it('should echo the request back', function (done) {
      var expected = {
        id: "foo",
        num: "0",
        status: null
      };
      client.EchoService.EchoService_EchoBody(
        { body: expected },
        { responseContentType: "application/json" }
      ).then(function (resp) {
        expect(resp.obj).toEqual(expected);
      }).catch(function (err) {
        done.fail(err);
      }).then(done);
    });
  });
});
