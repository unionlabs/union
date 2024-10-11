#!/usr/bin/env python3

"""
A simple script to parse the CSV output from the loadtime reporting tool (see
https://github.com/cometbft/cometbft/tree/main/test/loadtime/cmd/report).

Produces a plot of average transaction latency vs total transaction throughput
according to the number of load testing tool WebSocket connections to the
CometBFT node.
"""

import argparse
import csv
import logging
import sys
import matplotlib.pyplot as plt
import numpy as np


DEFAULT_TITLE = "CometBFT latency vs throughput"


def plot_latency_vs_throughput(input_files, output_image, title=DEFAULT_TITLE):
    avg_latencies, throughput_rates = process_input_files(input_files, )

    fig, ax = plt.subplots()

    connections = sorted(avg_latencies.keys())
    for c in connections:
        tr = np.array(throughput_rates[c])
        al = np.array(avg_latencies[c])
        label = '%d connection%s' % (c, '' if c == 1 else 's')
        ax.plot(tr, al, 'o-', label=label)

    ax.set_title(title)
    ax.set_xlabel('Throughput rate (tx/s)')
    ax.set_ylabel('Average transaction latency (s)')

    plt.legend(loc='upper left')
    plt.savefig(output_image)


def process_input_files(input_files):
    # Experimental data from which we will derive the latency vs throughput
    # statistics
    experiments = {}

    for input_file in input_files:
        logging.info('Reading %s...' % input_file)

        with open(input_file, 'rt') as inf:
            reader = csv.DictReader(inf)
            for tx in reader:
                experiments = process_tx(experiments, tx)

    return compute_experiments_stats(experiments)


def process_tx(experiments, tx):
    exp_id = tx['experiment_id']
    # Block time is nanoseconds from the epoch - convert to seconds
    block_time = float(tx['block_time']) / (10**9)
    # Duration is also in nanoseconds - convert to seconds
    duration = float(tx['duration_ns']) / (10**9)
    connections = int(tx['connections'])
    rate = int(tx['rate'])

    if exp_id not in experiments:
        experiments[exp_id] = {
            'connections': connections,
            'rate': rate,
            'block_time_min': block_time,
            # We keep track of the latency associated with the minimum block
            # time to estimate the start time of the experiment
            'block_time_min_duration': duration,
            'block_time_max': block_time,
            'total_latencies': duration,
            'tx_count': 1,
        }
        logging.info('Found experiment %s with rate=%d, connections=%d' %
                     (exp_id, rate, connections))
    else:
        # Validation
        for field in ['connections', 'rate']:
            val = int(tx[field])
            if val != experiments[exp_id][field]:
                raise Exception(
                    'Found multiple distinct values for field '
                    '"%s" for the same experiment (%s): %d and %d' %
                    (field, exp_id, val, experiments[exp_id][field]))

        if block_time < experiments[exp_id]['block_time_min']:
            experiments[exp_id]['block_time_min'] = block_time
            experiments[exp_id]['block_time_min_duration'] = duration
        if block_time > experiments[exp_id]['block_time_max']:
            experiments[exp_id]['block_time_max'] = block_time

        experiments[exp_id]['total_latencies'] += duration
        experiments[exp_id]['tx_count'] += 1

    return experiments


def compute_experiments_stats(experiments):
    """Compute average latency vs throughput rate statistics from the given
    experiments"""
    stats = {}

    # Compute average latency and throughput rate for each experiment
    for exp_id, exp in experiments.items():
        conns = exp['connections']
        avg_latency = exp['total_latencies'] / exp['tx_count']
        exp_start_time = exp['block_time_min'] - exp['block_time_min_duration']
        exp_duration = exp['block_time_max'] - exp_start_time
        throughput_rate = exp['tx_count'] / exp_duration
        if conns not in stats:
            stats[conns] = []

        stats[conns].append({
            'avg_latency': avg_latency,
            'throughput_rate': throughput_rate,
        })

    # Sort stats for each number of connections in order of increasing
    # throughput rate, and then extract average latencies and throughput rates
    # as separate data series.
    conns = sorted(stats.keys())
    avg_latencies = {}
    throughput_rates = {}
    for c in conns:
        stats[c] = sorted(stats[c], key=lambda s: s['throughput_rate'])
        avg_latencies[c] = []
        throughput_rates[c] = []
        for s in stats[c]:
            avg_latencies[c].append(s['avg_latency'])
            throughput_rates[c].append(s['throughput_rate'])
            logging.info('For %d connection(s): '
                         'throughput rate = %.6f tx/s\t'
                         'average latency = %.6fs' %
                         (c, s['throughput_rate'], s['avg_latency']))

    return (avg_latencies, throughput_rates)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="Renders a latency vs throughput diagram "
        "for a set of transactions provided by the loadtime reporting tool",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter)
    parser.add_argument('-t',
                        '--title',
                        default=DEFAULT_TITLE,
                        help='Plot title')
    parser.add_argument('output_image',
                        help='Output image file (in PNG format)')
    parser.add_argument(
        'input_csv_file',
        nargs='+',
        help="CSV input file from which to read transaction data "
        "- must have been generated by the loadtime reporting tool")
    args = parser.parse_args()

    logging.basicConfig(format='%(levelname)s\t%(message)s',
                        stream=sys.stdout,
                        level=logging.INFO)
    
    plot_latency_vs_throughput(args.input_csv_file, args.output_image, title=args.title)
