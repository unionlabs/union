#!/usr/bin/env python3

# Requirements:
# pip install requests matplotlib pandas prometheus-pandas
import os
import requests 
import sys

import matplotlib.pyplot as plt
import matplotlib.dates as md
import numpy as np 
import pandas as pd 

from urllib.parse import urljoin
from prometheus_pandas import query as prometheus_query


PROMETHEUS_URL = 'http://localhost:9090'
IMAGES_DIR = 'imgs'
TEST_CASES = ['200_nodes', 'rotating', 'vote_extensions']


def usage():
    print("Usage:")
    print(f"\t{sys.argv[0]} release_name start_time window_size test_case")
    print("where:")
    print(f"- start_time is a UTF time in '%Y-%m-%dT%H:%M:%SZ' format")
    print(f"- window size is in seconds")
    print(f"- test_case is one of {TEST_CASES}")
    print(f"Example: \t{sys.argv[0]} v1.0.0-alpha.2 2024-03-21T08:45:23Z 180 200_nodes")
    exit(1)


def queries_200_nodes(time_window, ext_time_window):
    return [
        (( 'cometbft_mempool_size',                           time_window[0], time_window[1], '1s'), 'mempool_size',              dict(ylabel='TXs',               xlabel='time (s)', title='Mempool Size',                   legend=False, figsize=(10,6), grid=True, ylim=(0, 5100), kind='area',stacked=True), False),
        (( 'avg(cometbft_mempool_size)',                      time_window[0], time_window[1], '1s'), 'avg_mempool_size',          dict(ylabel='TXs',               xlabel='time (s)', title='Average Mempool Size',           legend=False, figsize=(10,6), grid=True, ylim=(0, 5100)), False),
        (( 'max(cometbft_mempool_size)',                      time_window[0], time_window[1], '1s'), 'mempool_size_max',          dict(ylabel='TXs',               xlabel='time (s)', title='Maximum Mempool Size',           legend=False, figsize=(10,6), grid=True, ylim=(0, 5100)), False),
        (( 'cometbft_p2p_peers',                              time_window[0], time_window[1], '1s'), 'peers',                     dict(ylabel='# Peers',           xlabel='time (s)', title='Peers',                          legend=False, figsize=(10,6), grid=True, ylim=(0, 150)), True),
        #(( 'cometbft_consensus_height',                       time_window[0], time_window[1], '1s'), 'blocks_regular',            dict(ylabel='# Blocks',           xlabel='time (s)', title='Blocks in time',                 legend=False, figsize=(10,6), grid=True), False), 
        (( 'cometbft_consensus_rounds',                       time_window[0], time_window[1], '1s'), 'rounds',                    dict(ylabel='# Rounds',          xlabel='time (s)', title='Rounds per block',               legend=False, figsize=(10,6), grid=True, ylim=(0, 4)), False),
        (( 'rate(cometbft_consensus_height[20s])*60',         time_window[0], time_window[1], '1s'), 'block_rate_regular',        dict(ylabel='Blocks/min',        xlabel='time (s)', title='Rate of block creation',         legend=False, figsize=(10,6), grid=True, ylim=(0, 120)), True),
        #(( 'avg(rate(cometbft_consensus_height[20s])*60)',    time_window[0], time_window[1], '1s'), 'block_rate_avg_reg',        dict(ylabel='Blocks/min',        xlabel='time (s)', title='Rate of block creation',         legend=False, figsize=(10,6), grid=True), False),
        #(( 'cometbft_consensus_total_txs',                    time_window[0], time_window[1], '1s'), 'total_txs_regular',         dict(ylabel='# TXs',             xlabel='time (s)', title='Transactions in time',           legend=False, figsize=(10,6), grid=True), False),
        (( 'rate(cometbft_consensus_total_txs[20s])*60',      time_window[0], time_window[1], '1s'), 'total_txs_rate_regular',    dict(ylabel='TXs/min',           xlabel='time (s)', title='Rate of transaction processing', legend=False, figsize=(10,6), grid=True, ylim=(0, 50000)), True),
        #(( 'avg(rate(cometbft_consensus_total_txs[20s])*60)', time_window[0], time_window[1], '1s'), 'total_txs_rate_avg_reg',    dict(ylabel='TXs/min',           xlabel='time (s)', title='Rate of transaction processing', legend=False, figsize=(10,6), grid=True), False),
        (( 'process_resident_memory_bytes',                   time_window[0], time_window[1], '1s'), 'memory',                    dict(ylabel='Memory (bytes)',    xlabel='time (s)', title='Memory usage',                   legend=False, figsize=(10,6), grid=True, ylim=(0, 2e9)), False),
        (( 'avg(process_resident_memory_bytes)',              time_window[0], time_window[1], '1s'), 'avg_memory',                dict(ylabel='Memory (bytes)',    xlabel='time (s)', title='Average Memory usage',           legend=False, figsize=(10,6), grid=True, ylim=(0, 2e9)), False),
        (( 'node_load1',                                      time_window[0], time_window[1], '1s'), 'cpu',                       dict(ylabel='Load',              xlabel='time (s)', title='Node load',                      legend=False, figsize=(10,6), grid=True, ylim=(0, 6)), False), 
        (( 'avg(node_load1)',                                 time_window[0], time_window[1], '1s'), 'avg_cpu',                   dict(ylabel='Load',              xlabel='time (s)', title='Average Node load',              legend=False, figsize=(10,6), grid=True, ylim=(0, 6)), False),
        (( 'cometbft_consensus_block_size_bytes/1024/1024',   time_window[0], time_window[1], '1s'), 'block_size_bytes',          dict(ylabel='Mb',                xlabel='time (s)', title='Block size (Mb)',                legend=False, figsize=(10,6), grid=True, ylim=(0, 4.1)), False),
        
        # Extended window metrics
        (( 'cometbft_consensus_height',                       ext_time_window[0], ext_time_window[1], '1s'), 'blocks',            dict(ylabel='# Blocks',          xlabel='time (s)', title='Blocks in time',                 legend=False, figsize=(10,6), grid=True), False), 
        (( 'rate(cometbft_consensus_height[20s])*60',         ext_time_window[0], ext_time_window[1], '1s'), 'block_rate',        dict(ylabel='Blocks/min',        xlabel='time (s)', title='Rate of block creation',         legend=False, figsize=(10,6), grid=True), True),
        (( 'cometbft_consensus_total_txs',                    ext_time_window[0], ext_time_window[1], '1s'), 'total_txs',         dict(ylabel='# TXs',             xlabel='time (s)', title='Transactions in time',           legend=False, figsize=(10,6), grid=True), False),
        (( 'rate(cometbft_consensus_total_txs[20s])*60',      ext_time_window[0], ext_time_window[1], '1s'), 'total_txs_rate',    dict(ylabel='TXs/min',           xlabel='time (s)', title='Rate of transaction processing', legend=False, figsize=(10,6), grid=True, ylim=(0, 50000)), True),
    ]


def queries_rotating(time_window):
    return [
        (( 'rate(cometbft_consensus_height[20s])*60<1000>0', time_window[0], time_window[1], '1s'), 'rotating_block_rate',  dict(ylabel='blocks/min',     xlabel='time', title='Rate of Block Creation',         legend=False, figsize=(10,6), grid=True), False),
        (( 'rate(cometbft_consensus_total_txs[20s])*60',     time_window[0], time_window[1], '1s'), 'rotating_txs_rate',    dict(ylabel='TXs/min',        xlabel='time', title='Rate of Transaction processing', legend=False, figsize=(10,6), grid=True), False),
        (( 'cometbft_consensus_height{job=~"ephemeral.*"}>cometbft_blocksync_latest_block_height{job=~"ephemeral.*"} or cometbft_blocksync_latest_block_height{job=~"ephemeral.*"}',
                                                             time_window[0], time_window[1], '1s'), 'rotating_eph_heights', dict(ylabel='height',         xlabel='time', title='Heights of Ephemeral Nodes',     legend=False, figsize=(10,6), grid=True), False),
        (( 'cometbft_p2p_peers',                             time_window[0], time_window[1], '1s'), 'rotating_peers',       dict(ylabel='# peers',        xlabel='time', title='Peers',                          legend=False, figsize=(10,6), grid=True), False),
        (( 'avg(process_resident_memory_bytes)',             time_window[0], time_window[1], '1s'), 'rotating_avg_memory',  dict(ylabel='memory (bytes)', xlabel='time', title='Average Memory Usage',           legend=False, figsize=(10,6), grid=True), False),
        (( 'node_load1',                                     time_window[0], time_window[1], '1s'), 'rotating_cpu',         dict(ylabel='load',           xlabel='time', title='Node Load',                      legend=False, figsize=(10,6), grid=True), False),
    ]


def queries_vote_extensions(time_window):
    return [
        (( 'cometbft_mempool_size',                      time_window[0], time_window[1], '1s'), 'mempool_size',              dict(ylabel='TXs',               xlabel='time (s)', title='Mempool Size',                   legend=False, figsize=(10,6), grid=True, kind='area',stacked=True), False),
        (( 'cometbft_mempool_size',                      time_window[0], time_window[1], '1s'), 'mempool_size_not_stacked',  dict(ylabel='TXs',               xlabel='time (s)', title='Mempool Size',                   legend=False, figsize=(10,6), grid=True, stacked=False), False),
        (( 'cometbft_p2p_peers',                         time_window[0], time_window[1], '1s'), 'peers',                     dict(ylabel='# Peers',           xlabel='time (s)', title='Peers',                          legend=False, figsize=(10,6), grid=True), True),
        (( 'avg(cometbft_mempool_size)',                 time_window[0], time_window[1], '1s'), 'avg_mempool_size',          dict(ylabel='TXs',               xlabel='time (s)', title='Average Mempool Size',           legend=False, figsize=(10,6), grid=True), False),
        (( 'cometbft_consensus_rounds',                  time_window[0], time_window[1], '1s'), 'rounds',                    dict(ylabel='# Rounds',          xlabel='time (s)', title='Rounds per block',               legend=False, figsize=(10,6), grid=True), False),
        (( 'process_resident_memory_bytes',              time_window[0], time_window[1], '1s'), 'memory',                    dict(ylabel='Memory (bytes)',    xlabel='time (s)', title='Memory usage',                   legend=False, figsize=(10,6), grid=True), False),
        (( 'avg(process_resident_memory_bytes)',         time_window[0], time_window[1], '1s'), 'avg_memory',                dict(ylabel='Memory (bytes)',    xlabel='time (s)', title='Average Memory usage',           legend=False, figsize=(10,6), grid=True), False),
        (( 'node_load1',                                 time_window[0], time_window[1], '1s'), 'cpu',                       dict(ylabel='Load',              xlabel='time (s)', title='Node load',                      legend=False, figsize=(10,6), grid=True), False), 
        (( 'avg(node_load1)',                            time_window[0], time_window[1], '1s'), 'avg_cpu',                   dict(ylabel='Load',              xlabel='time (s)', title='Average Node load',              legend=False, figsize=(10,6), grid=True), False),
        (( 'cometbft_consensus_height',                  time_window[0], time_window[1], '1s'), 'blocks',                    dict(ylabel='# Blocks',          xlabel='time (s)', title='Blocks in time',                 legend=False, figsize=(10,6), grid=True), False), 
        (( 'rate(cometbft_consensus_height[20s])*60',    time_window[0], time_window[1], '1s'), 'block_rate',                dict(ylabel='Blocks/min',        xlabel='time (s)', title='Rate of block creation',         legend=False, figsize=(10,6), grid=True), True),
        (( 'cometbft_consensus_total_txs',               time_window[0], time_window[1], '1s'), 'total_txs',                 dict(ylabel='# TXs',             xlabel='time (s)', title='Transactions in time',           legend=False, figsize=(10,6), grid=True), False),
        (( 'rate(cometbft_consensus_total_txs[20s])*60', time_window[0], time_window[1], '1s'), 'total_txs_rate',            dict(ylabel='TXs/min',           xlabel='time (s)', title='Rate of transaction processing', legend=False, figsize=(10,6), grid=True), True),
    ]


def main(release, start_time, window_size, test_case):
    prometheus = prometheus_query.Prometheus(PROMETHEUS_URL)

    end_time = pd.to_datetime(start_time) + pd.Timedelta(**dict(seconds=window_size))
    time_window = (start_time, end_time.strftime('%Y-%m-%dT%H:%M:%SZ'))

    ext_end_time = pd.to_datetime(start_time) + pd.Timedelta(**dict(seconds=window_size+50))
    ext_time_window = (start_time, ext_end_time.strftime('%Y-%m-%dT%H:%M:%SZ'))

    # Select queries depending on the test case.
    match test_case:
        case "200_nodes": 
            queries = queries_200_nodes(time_window, ext_time_window)
        case "rotating": 
            queries = queries_rotating(time_window)
        case "vote_extensions": 
            queries = queries_vote_extensions(time_window)
        case _:
            print(f"Error: Unknown test case {test_case}")
            return

    imgs_dir = os.path.join(IMAGES_DIR, test_case)
    if not os.path.exists(imgs_dir):
        os.makedirs(imgs_dir)

    # Query Prometheus and plot images.
    for (query, file_name, pandas_params, plot_average) in queries:
        print(f"query: {query}")

        df = prometheus.query_range(*query)
        #Tweak the x ticks
        df = df.set_index(md.date2num(df.index))

        if df.empty:
            print('No data found! Check the timestamps or the query.')
            continue
        
        pandas_params["title"] += "  -  " + release
        ax = df.plot(**pandas_params)
        if plot_average:
            average = df.mean(axis=1)
            df['__average__'] = average
            pandas_params['lw'] = 8
            pandas_params['style'] = ['--']
            pandas_params['color'] = ['red']
            ax = df['__average__'].plot(**pandas_params)

        ax.xaxis.set_major_formatter(md.DateFormatter('%H:%M:%S'))
        plt.savefig(os.path.join(imgs_dir, file_name + '.png'))
        plt.plot()

    plt.show()


if __name__ == "__main__":
    if len(sys.argv) < 5 or not (sys.argv[1] and sys.argv[2] and sys.argv[3] and sys.argv[4]):
        usage()

    release = sys.argv[1]
    start_time = sys.argv[2]
    window_size = sys.argv[3]
    test_case = sys.argv[4]
    main(release, start_time, int(window_size), test_case)
