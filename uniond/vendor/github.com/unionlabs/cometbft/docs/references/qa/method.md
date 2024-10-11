---
order: 1
parent:
  title: Method
  order: 1
---

# Method

This document provides a detailed description of the QA process.
It is intended to be used by engineers reproducing the experimental setup for future tests of CometBFT.

The (first iteration of the) QA process as described [in the RELEASES.md document][releases]
was applied to version v0.34.x in order to have a set of results acting as benchmarking baseline.
This baseline is then compared with results obtained in later versions.
See [RELEASES.md][releases] for a description of the tests that we run on the QA process.

Out of the testnet-based test cases described in [the releases document][releases] we focused on two of them:
_200 Node Test_, and _Rotating Nodes Test_.

[releases]: https://github.com/cometbft/cometbft/blob/main/RELEASES.md#large-scale-testnets

## Table of Contents
- [Method](#method)
  - [Table of Contents](#table-of-contents)
  - [Software Dependencies](#software-dependencies)
    - [Infrastructure Requirements to Run the Tests](#infrastructure-requirements-to-run-the-tests)
    - [Requirements for Result Extraction](#requirements-for-result-extraction)
  - [200 Node Testnet](#200-node-testnet)
    - [Running the test](#running-the-test)
    - [Result Extraction](#result-extraction)
      - [Steps](#steps)
      - [Extracting Prometheus Metrics](#extracting-prometheus-metrics)
  - [Rotating Node Testnet](#rotating-node-testnet)
    - [Running the test](#running-the-test-1)
    - [Result Extraction](#result-extraction-1)
  - [Vote Extensions Testnet](#vote-extensions-testnet)
    - [Running the test](#running-the-test-2)
    - [Result Extraction](#result-extraction-2)

## Software Dependencies

### Infrastructure Requirements to Run the Tests

* An account at Digital Ocean (DO), with a high droplet limit (>202)
* The machine to orchestrate the tests should have the following installed:
    * A clone of the [testnet repository][testnet-repo]
        * This repository contains all the scripts mentioned in the remainder of this section
    * [Digital Ocean CLI][doctl]
    * [Terraform CLI][Terraform]
    * [Ansible CLI][Ansible]

[testnet-repo]: https://github.com/cometbft/qa-infra
[Ansible]: https://docs.ansible.com/ansible/latest/index.html
[Terraform]: https://www.terraform.io/docs
[doctl]: https://docs.digitalocean.com/reference/doctl/how-to/install/

### Requirements for Result Extraction

* [Prometheus DB][prometheus] to collect metrics from nodes
* Prometheus DB to process queries (may be different node from the previous)
* blockstore DB of one of the full nodes in the testnet


[prometheus]: https://prometheus.io/

## 200 Node Testnet

This test consists in spinning up 200 nodes (175 validators + 20 full nodes + 5 seed nodes) and
performing two experiments:
- First we find the [saturation point](saturation) of the network by running the script
  [200-node-loadscript.sh][200-node-loadscript.sh].
- Then we run several times the testnet using the saturation point to collect data.

The script [200-node-loadscript.sh] runs multiple transaction load instances with all possible
combinations of the following parameters:
- number of transactions sent per second (the rate): 200, 400, 800, and 1600.
- number of connections to the target node: 1, 2, and 4.

Additionally:
- The size of each transaction is 1024 bytes.
- The duration of each test is 90 seconds.
- There is one target node (a validator) that receives all the load.
- After each test iteration, it waits that the mempool is empty and then wait `120 + rate /60`
  seconds more.

[200-node-loadscript.sh]: https://github.com/cometbft/qa-infra/blob/main/ansible/scripts/200-node-loadscript.sh
[saturation]: CometBFT-QA-34.md#saturation-point

### Running the test

This section explains how the tests were carried out for reproducibility purposes.

1. [If you haven't done it before]
   Follow steps 1-4 of the `README.md` at the top of the testnet repository to configure Terraform and the DigitalOcean CLI (`doctl`).
2. In the `experiment.mk` file, set the following variables (do NOT commit these changes):
   1. Set `MANIFEST` to point to the file `testnets/200-nodes-with-zones.toml`.
   2. Set `VERSION_TAG` to the git hash that is to be tested.
      * If you are running the base test, which implies an homogeneous network (all nodes are running the same version),
        then make sure makefile variable `VERSION2_WEIGHT` is set to 0
      * If you are running a mixed network, set the variable `VERSION2_TAG` to the other version you want deployed
        in the network.
        Then adjust the weight variables `VERSION_WEIGHT` and `VERSION2_WEIGHT` to configure the
        desired proportion of nodes running each of the two configured versions.
3. Follow steps 5-11 of the `README.md` to configure and start the 200 node testnet.
    * WARNING: Do NOT forget to run `make terraform-destroy` as soon as you are done with the tests (see step 9)
4. As a sanity check, connect to the Prometheus node's web interface (port 9090)
    and check the graph for the `cometbft_consensus_height` metric. All nodes
    should be increasing their heights.

    * Run `ansible --list-hosts prometheus` to obtain the Prometheus node's IP address.
    * The following URL will display the metrics `cometbft_consensus_height` and `cometbft_mempool_size`:

      ```
      http://<PROMETHEUS-NODE-IP>:9090/classic/graph?g0.range_input=1h&g0.expr=cometbft_consensus_height&g0.tab=0&g1.range_input=1h&g1.expr=cometbft_mempool_size&g1.tab=0
      ```

5. Discover the saturation point of the network. If you already know it, skip this step.
  * Run `make loadrunners-init`, in case the load runner is not yet initialised. This will copy the
    loader scripts to the `testnet-load-runner` node and install the load tool.
  * Run `ansible --list-hosts loadrunners` to find the IP address of the `testnet-load-runner` node.
  * `ssh` into `testnet-load-runner`.
    * We will run a script that takes about 40 mins to complete, so it is suggested to first run `tmux` in case the ssh session breaks.
        * `tmux` quick cheat sheet: `ctrl-b a` to attach to an existing session; `ctrl-b %` to split the current pane vertically; `ctrl-b ;` to toggle to last active pane.
    * Find the *internal* IP address of a full node (for example, `validator000`). This node will receive all transactions from the load runner node.
    * Run `/root/200-node-loadscript.sh <INTERNAL_IP>` from the load runner node, where `<INTERNAL_IP>` is the internal IP address of a full node.
      * The script runs 90-seconds-long experiments in a loop with different load values.
    * Follow the steps of the [Result Extraction](#result-extraction) section below to obtain the file `report_tabbed.txt`.

6. Run several transaction load instances (typically 5), each of 90 seconds, using a load somewhat below the saturation point.
  * Set the Makefile variables `LOAD_CONNECTIONS`, `LOAD_TX_RATE`, to values that will produce the desired transaction load.
  * Set `LOAD_TOTAL_TIME` to 91 (seconds). The extra second is because the last transaction batch
    coincides with the end of the experiment and is thus not sent.
  * Run `make runload` and wait for it to complete. You may want to run this several times so the data from different runs can be compared.
7. Run `make retrieve-data` to gather all relevant data from the testnet into the orchestrating machine
    * Alternatively, you may want to run `make retrieve-prometheus-data` and `make retrieve-blockstore` separately.
      The end result will be the same.
    * `make retrieve-blockstore` accepts the following values in makefile variable `RETRIEVE_TARGET_HOST`
        * `any`: (which is the default) picks up a full node and retrieves the blockstore from that node only.
        * `all`: retrieves the blockstore from all full nodes; this is extremely slow, and consumes plenty of bandwidth,
           so use it with care.
        * the name of a particular full node (e.g., `validator01`): retrieves the blockstore from that node only.
8. Verify that the data was collected without errors
    * at least one blockstore DB for a CometBFT validator
    * the Prometheus database from the Prometheus node
    * for extra care, you can run `zip -T` on the `prometheus.zip` file and (one of) the `blockstore.db.zip` file(s)
9.  **Run `make terraform-destroy`**
    * Don't forget to type `yes`! Otherwise you're in trouble.

### Result Extraction

The method for extracting the results described here is highly manual (and exploratory) at this stage.
The CometBFT team should improve it at every iteration to increase the amount of automation.

#### Saturation point

For identifying the saturation point, run from the `qa-infra` repository:
```sh
./script/reports/saturation-gen-table.sh <experiments-blockstore-dir>
```
where `<experiments-blockstore-dir>` is the directory where the results of the experiments were downloaded.
This directory should contain the file `blockstore.db.zip`. The script will automatically:
1. Unzip `blockstore.db.zip`, if not already.
2. Run the tool `test/loadtime/cmd/report` to extract data for all instances with different
   transaction load.
  - This will generate an intermediate file `report.txt` that contains an unordered list of
    experiments results with varying concurrent connections and transaction rate.
3. Generate the files:
  -  `report_tabbed.txt` with results formatted as a matrix, where rows are a particular tx rate and
     columns are a particular number of websocket connections.
  -  `saturation_table.tsv` which just contains columns with the number of processed transactions;
     this is handy to create a Markdown table for the report.

#### Latencies

For generating images on latency, run from the `qa-infra` repository:
```sh
./script/reports/latencies-gen-images.sh <experiments-blockstore-dir>
```
As above, `<experiments-blockstore-dir>` should contain the file `blockstore.db.zip`. 
The script will automatically:
1. Unzip `blockstore.db.zip`, if not already.
2. Generate a file with raw results `results/raw.csv` using the tool `test/loadtime/cmd/report`.
3. Setup a Python virtual environment and install the dependencies required for running the scripts
   in the steps below.
4. Generate a latency vs throughput images, using [`latency_throughput.py`]. This plot is useful to
   visualize the saturation point.
5. Generate a series of images with the average latency of each block for each experiment instance
   and configuration, using [`latency_plotter.py`]. This plots may help with visualizing latency vs.
   throughput variation.

[`latency_throughput.py`]: ../../../scripts/qa/reporting/README.md#Latency-vs-Throughput-Plotting
[`latency_plotter.py`]: ../../../scripts/qa/reporting/README.md#Latency-vs-Throughput-Plotting-version-2

#### Prometheus metrics

1. From the `qa-infra` repository, run:
    ```sh
    ./script/reports/prometheus-start-local.sh <experiments-prometheus-dir>
    ```
    where `<experiments-prometheus-dir>` is the directory where the results of the experiments were
    downloaded. This directory should contain the file `blockstore.db.zip`. This script will:
    - kill any running Prometheus server,
    - unzip the Prometheus database retrieved from the testnet, and
    - start a Prometheus server on the default `localhost:9090`, bootstrapping the downloaded data
      as database.
2. Identify the time window you want to plot in your graphs. In particular, search for the start
   time and duration of the window.
3. Run:
    ```sh
    ./script/reports/prometheus-gen-images.sh <experiments-prometheus-dir> <start-time> <duration> [<test-case>] [<release-name>]
    ```
    where `<start-time>` is in the format `'%Y-%m-%dT%H:%M:%SZ'` and `<duration>` is in seconds.
    This will download, set up a Python virtual environment with required dependencies, and execute
    the script [`prometheus_plotter.py`]. The optional parameter `<test-case>` is one of `200_nodes`
    (default), `rotating`, and `vote_extensions`; `<release-name>` is just for putting in the title
    of the plot.

[`prometheus_plotter.py`]: ../../../scripts/qa/reporting/README.md#prometheus-metrics

## Rotating Node Testnet

### Running the test

This section explains how the tests were carried out for reproducibility purposes.

1. [If you haven't done it before]
   Follow the [set up][qa-setup] steps of the `README.md` at the top of the testnet repository to
   configure Terraform, and `doctl`.
2. In the `experiment.mk` file, set the following variables (do NOT commit these changes):
  * Set `MANIFEST` to point to the file `testnets/rotating.toml`.
  * Set `VERSION_TAG` to the git hash that is to be tested.
  * Set `EPHEMERAL_SIZE` to 25.
3. Follow the [testnet starting][qa-start] steps of the `README.md` to configure and start the
   the rotating node testnet.
  * WARNING: Do NOT forget to run `make terraform-destroy` as soon as you are done with the tests.
4. As a sanity check, connect to the Prometheus node's web interface and check the graph for the
   `cometbft_consensus_height` metric. All nodes should be increasing their heights.
5. On a different shell,
  * run `make loadrunners-init` to initialize the load runner.
  * run `make runload ITERATIONS=1 LOAD_CONNECTIONS=X LOAD_TX_RATE=Y LOAD_TOTAL_TIME=Z`
    * `X` and `Y` should reflect a load below the saturation point (see, e.g.,
      [this paragraph](CometBFT-QA-38#saturation-point) for further info)
    * `Z` (in seconds) should be big enough to keep running throughout the test, until we manually stop it in step 7.
      In principle, a good value for `Z` is `7200` (2 hours)
6. Run `make rotate` to start the script that creates the ephemeral nodes, and kills them when they are caught up.
  * WARNING: If you run this command from your laptop, the laptop needs to be up and connected for the full length
    of the experiment.
  * [This][rotating-prometheus] is an example Prometheus URL you can use to monitor the test case's progress.
7. When the height of the chain reaches 3000, stop the `make runload` script.
8. When the rotate script has made two iterations (i.e., all ephemeral nodes have caught up twice)
   after height 3000 was reached, stop `make rotate`.
9. Run `make stop-network`.
10. Run `make retrieve-data` to gather all relevant data from the testnet into the orchestrating machine
11. Verify that the data was collected without errors
  * at least one blockstore DB for a CometBFT validator
  * the Prometheus database from the Prometheus node
  * for extra care, you can run `zip -T` on the `prometheus.zip` file and (one of) the `blockstore.db.zip` file(s)
12.  **Run `make terraform-destroy`**

Steps 8 to 10 are highly manual at the moment and will be improved in next iterations.

### Result Extraction

In order to obtain a latency plot, follow the instructions above for the 200 node experiment,
but the `results.txt` file contains only one experiment.

As for prometheus, the same method as for the 200 node experiment can be applied.

## Vote Extensions Testnet

### Running the test

This section explains how the tests were carried out for reproducibility purposes.

1. [If you haven't done it before]
   Follow the [set up][qa-setup] steps of the `README.md` at the top of the testnet repository to
   configure Terraform, and `doctl`.
2. In the `experiment.mk` file, set the following variables (do NOT commit these changes):
   1. Set `MANIFEST` to point to the file `testnets/varyVESize.toml`.
   2. Set `VERSION_TAG` to the git hash that is to be tested.
3. Follow the [testnet starting][qa-start] steps of the `README.md` to configure and start
   the testnet.
    * WARNING: Do NOT forget to run `make terraform-destroy` as soon as you are done with the tests
4. Configure the load runner to produce the desired transaction load.
    * set makefile variables `ROTATE_CONNECTIONS`, `ROTATE_TX_RATE`, to values that will produce the desired transaction load.
    * set `ROTATE_TOTAL_TIME` to 150 (seconds).
    * set `ITERATIONS` to the number of iterations that each configuration should run for.
5. Execute the [testnet starting][qa-start] steps of the `README.md` file at the testnet repository.
6. Repeat the following steps for each desired `vote_extension_size`
    1. Update the configuration (you can skip this step if you didn't change the `vote_extension_size`)
        * Update the `vote_extensions_size` in the `testnet.toml` to the desired value.
        * `make configgen`
        * `ANSIBLE_SSH_RETRIES=10 ansible-playbook ./ansible/re-init-testapp.yaml -u root -i ./ansible/hosts --limit=validators -e "testnet_dir=testnet" -f 20`
        * `make restart`
    2. Run the test
        * `make runload`
          This will repeat the tests `ITERATIONS` times every time it is invoked.
    3. Collect your data
        * `make retrieve-data`
          Gathers all relevant data from the testnet into the orchestrating machine, inside folder `experiments`.
          Two subfolders are created, one blockstore DB for a CometBFT validator and one for the Prometheus DB data.
        * Verify that the data was collected without errors with `zip -T` on the `prometheus.zip` file and (one of) the `blockstore.db.zip` file(s).
7. Clean up your setup.
    * `make terraform-destroy`; don't forget that you need to type **yes** for it to complete.


### Result Extraction

In order to obtain a latency plot, follow the instructions above for the 200 node experiment, but:

* The `results.txt` file contains only one experiment
* Therefore, no need for any `for` loops

As for Prometheus, the same method as for the 200 node experiment can be applied.

[qa-setup]: https://github.com/cometbft/qa-infra/blob/main/README.md#setup
[qa-start]: https://github.com/cometbft/qa-infra/blob/main/README.md#start-the-network
[rotating-prometheus]: http://PROMETHEUS-NODE-IP:9090/classic/graph?g0.expr=cometbft_consensus_height%7Bjob%3D~%22ephemeral.*%22%7D%3Ecometbft_blocksync_latest_block_height%7Bjob%3D~%22ephemeral.*%22%7D%20or%20cometbft_blocksync_latest_block_height%7Bjob%3D~%22ephemeral.*%22%7D&g0.tab=0&g0.display_mode=lines&g0.show_exemplars=0&g0.range_input=1h40m&g1.expr=cometbft_mempool_size%7Bjob!~%22ephemeral.*%22%7D&g1.tab=0&g1.display_mode=lines&g1.show_exemplars=0&g1.range_input=1h40m&g2.expr=cometbft_consensus_num_txs%7Bjob!~%22ephemeral.*%22%7D&g2.tab=0&g2.display_mode=lines&g2.show_exemplars=0&g2.range_input=1h40m
