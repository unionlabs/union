## Token Transfer Model-based Testing Guide

In the process of IBC Audit performed by Informal Systems, we have implemented 
a preliminary set of model-based tests for the ICS-20 Token Transfer implementation.

Model-based tests are based on the formal `TLA+` model of the Token transfer relay functions: see [relay.tla](relay_model/relay.tla).
The tests themselves are simple `TLA+` assertions, that describe the desired shape of execution that send or receive tokens; 
see [relay_tests.tla](relay_model/relay_tests.tla) for some examples. 
To be able to specify test assertions the TLA+ model contains the `history` variable, 
which records the whole execution history. 
So, by way of referring to `history` you simply specify declaratively what execution history you want to see.

After you have specified your `TLA+` test, you can run it using [Apalache model checker](https://github.com/informalsystems/apalache).
E.g. for the test `TestUnescrowTokens` run 

```bash
apalache-mc check --inv=TestUnescrowTokensInv relay_tests.tla
```
 
In case there are no error in the TLA+ model or in the test assertions, this will produce a couple of so-called _counterexamples_. 
This is a terminology from the model-checking community; for the testing purposes they can be considered simply as model executions.
See the files `counterexample.tla` for human-readable representation, and `counterexample.json` for machine-readable one.

In order to execute the produced test, you need to translate it into another format. 
For that translation you need the tool [Jsonatr (JSON Arrifact Translator)](https://github.com/informalsystems/jsonatr). 
It performs the translation using this [transformation spec](relay_model/apalache-to-relay-test2.json);

To transform a counterexample into a test, run 

```bash
jsonatr --use apalache-to-relay-test2.json --in counterexample.json --out model_based_tests/YourTestName.json
```

Now, if you run `go test` in this directory, the file you have produced above should be picked up by the [model-based test driver](mbt_relay_test.go),
and executed automatically.


The easiest way to run Apalache is by 
[using a Docker image](https://apalache.informal.systems/docs/apalache/installation/docker.html); 
to run Jsonatr you need to locally clone the repository, and then, 
after building it, add the `target/debug` directory into your `PATH`. 

To wrap Apalache docker image into an executable you might create the following executable bash script `apalache-mc`:

```bash
#!/bin/bash
docker run --rm -v $(pwd):/var/apalache apalache/mc $@
```    


In case of any questions please don't hesitate to contact Andrey Kuprianov (andrey@informal.systems).
