#!/usr/bin/env python3

import sys
import os
import pytz
from datetime import datetime

import matplotlib as mpl
import matplotlib.pyplot as plt
import numpy as np
import pandas as pd

IMAGES_DIR = 'imgs'
#fig_title = 'Vote Extensions Testnet'
fig_title = 'Rotating Nodes Test'

def usage():
    print(f"Usage: {sys.argv[0]} release_name raw_csv_path")
    exit(1)


#FIXME: figure out in which timezone prometheus was running to adjust to UTC.
tz = pytz.timezone('UTC')


def plot_all_experiments(release, csv):
    # Group by experiment
    groups = csv.groupby(['experiment_id'])

    # number of rows and columns in the graph
    ncols = 2 if groups.ngroups > 1 else 1
    nrows = int( np.ceil(groups.ngroups / ncols)) if groups.ngroups > 1 else 1
    fig, axes = plt.subplots(nrows=nrows, ncols=ncols, figsize=(6*ncols, 4*nrows), sharey=False)
    fig.tight_layout(pad=5.0)

    # Plot experiments as subplots 
    for (key,ax) in zip(groups.groups.keys(), [axes] if ncols == 1 else axes.flatten()):
        group = groups.get_group(key)
        ax.set_ylabel('latency (s)')
        ax.set_xlabel('experiment time (s)')
        ax.set_title(key)
        ax.grid(True)

        # Group by connection number and transaction rate
        paramGroups = group.groupby(['connections','rate'])
        for (subKey) in paramGroups.groups.keys():
            subGroup = paramGroups.get_group(subKey)
            startTime = subGroup.block_time.min()
            endTime = subGroup.block_time.max()
            subGroup.block_time = subGroup.block_time.apply(lambda x: x - startTime )
            mean = subGroup.duration_ns.mean()
            localStartTime = tz.localize(datetime.fromtimestamp(startTime)).astimezone(pytz.utc)
            localEndTime  = tz.localize(datetime.fromtimestamp(endTime)).astimezone(pytz.utc)
            print('experiment', key ,'start', localStartTime.strftime("%Y-%m-%dT%H:%M:%SZ"), 'end', localEndTime.strftime("%Y-%m-%dT%H:%M:%SZ"), 'duration', endTime - startTime, "mean", mean)

            (con,rate) = subKey
            label = 'c='+str(con) + ' r='+ str(rate)
            ax.axhline(y = mean, color = 'r', linestyle = '-', label="mean")
            ax.scatter(subGroup.block_time, subGroup.duration_ns, label=label)
        ax.legend()

        # Save individual axes
        extent = ax.get_window_extent().transformed(fig.dpi_scale_trans.inverted())
        img_path = os.path.join(IMAGES_DIR, f'e_{key}.png')
        fig.savefig(img_path, bbox_inches=extent.expanded(1.4, 1.5))

    fig.suptitle(fig_title + ' - ' + release)

    # Save the figure with subplots
    fig.savefig(os.path.join(IMAGES_DIR, 'all_experiments.png'))


def plot_all_configs(release, csv):
    # Group by configuration
    groups = csv.groupby(['connections','rate'])

    # number of rows and columns in the graph
    ncols = 2 if groups.ngroups > 1 else 1
    nrows = int( np.ceil(groups.ngroups / ncols)) if groups.ngroups > 1 else 1
    fig, axes = plt.subplots(nrows=nrows, ncols=ncols, figsize=(6*ncols, 4*nrows), sharey=True)
    fig.tight_layout(pad=5.0)

    # Plot configurations as subplots 
    for (key,ax) in zip(groups.groups.keys(), [axes] if ncols == 1 else axes.flatten()):
        group = groups.get_group(key)
        ax.set_ylabel('latency (s)')
        ax.set_xlabel('experiment time (s)')
        ax.grid(True)
        (con,rate) = key
        label = 'c='+str(con) + ' r='+ str(rate)
        ax.set_title(label)

        # Group by experiment 
        paramGroups = group.groupby(['experiment_id'])
        for (subKey) in paramGroups.groups.keys():
            subGroup = paramGroups.get_group((subKey,))
            startTime = subGroup.block_time.min()
            subGroupMod = subGroup.block_time.apply(lambda x: x - startTime)
            ax.scatter(subGroupMod, subGroup.duration_ns, label=label)
        #ax.legend()
        

        #Save individual axes
        extent = ax.get_window_extent().transformed(fig.dpi_scale_trans.inverted())
        img_path = os.path.join(IMAGES_DIR, f'c{con}r{rate}.png')
        fig.savefig(img_path, bbox_inches=extent.expanded(1.4, 1.5))

    fig.suptitle(fig_title + ' - ' + release)

    # Save the figure with subplots
    fig.savefig(os.path.join(IMAGES_DIR, 'all_configs.png'))


def plot_merged(release, csv):
    # Group by configuration
    groups = csv.groupby(['connections','rate'])

    # number of rows and columns in the graph
    ncols = 2 if groups.ngroups > 1 else 1
    nrows = int( np.ceil(groups.ngroups / ncols)) if groups.ngroups > 1 else 1
    fig, axes = plt.subplots(nrows=nrows, ncols=ncols, figsize=(6*ncols, 4*nrows), sharey=True)
    fig.tight_layout(pad=5.0)

    # Plot configurations as subplots 
    for (key,ax) in zip(groups.groups.keys(), [axes] if ncols == 1 else axes.flatten()):
        group = groups.get_group(key)
        ax.set_ylabel('latency (s)')
        ax.set_xlabel('experiment time (s)')
        ax.grid(True)
        (con,rate) = key
        label = 'c='+str(con) + ' r='+ str(rate)
        ax.set_title(label)

        # Group by experiment, but merge them as a single experiment
        paramGroups = group.groupby(['experiment_id'])
        for (subKey) in paramGroups.groups.keys():
            subGroup = paramGroups.get_group((subKey,))
            startTime = subGroup.block_time.min()
            subGroupMod = subGroup.block_time.apply(lambda x: x - startTime)
            ax.scatter(subGroupMod, subGroup.duration_ns, marker='o',c='#1f77b4')
        
        # Save individual axes
        extent = ax.get_window_extent().transformed(fig.dpi_scale_trans.inverted())
        (con, rate) = key
        img_path = os.path.join(IMAGES_DIR, f'c{con}r{rate}_merged.png')
        fig.savefig(img_path, bbox_inches=extent)

    plt.show()


if __name__ == "__main__":
    if len(sys.argv) < 2 or not (sys.argv[1] and sys.argv[2]):
        usage()
    release = sys.argv[1]
    csv_path = sys.argv[2]

    if not os.path.exists(csv_path):
        print('Please provide a valid raw.csv file')
        exit()
    csv = pd.read_csv(csv_path)

    # Transform ns to s in the latency/duration
    csv['duration_ns'] = csv['duration_ns'].apply(lambda x: x/10**9)
    csv['block_time'] = csv['block_time'].apply(lambda x: x/10**9)

    if not os.path.exists(IMAGES_DIR):
        os.makedirs(IMAGES_DIR)

    plot_all_experiments(release, csv)
    plot_all_configs(release, csv)
    plot_merged(release, csv)
