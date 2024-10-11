#/usr/bin/env python3
"""Plotter for comparing saturation results on v1 and v0.38.

This script generates an image with the number of processed transactions for different load
configurations (tx rate and number of connections). The purpose is to find the saturation point of
the network and to compare the results between different CometBFT versions.

Quick setup before running:
```
python3 -m venv .venv && source .venv/bin/activate
pip install matplotlib
```
"""
import matplotlib.pyplot as plt
import numpy as np

# Expected values of processed transactions for a given transaction rate.
rates0 = [0, 3600]
expected = [r * 89 for r in rates0]

# Transaction rate (x axis)
rates1 = [200, 400, 800, 1600]
rates2 = [r*2 for r in rates1]
rates4 = [r*2 for r in rates2]

# v1 (without latency emulation), for number of connections c in [1,2,4]
c1 = [17800,31200,51146,50889]
c2 = [34600,54706,51917,47732]
c4 = [50464,49463,41376,45530]

# v0.38, for number of connections c in [1,2,4]
d1 = [17800,35600,36831,40600]
d2 = [33259,41565,38686,45034]
d4 = [33259,41384,40816,39830]

fig, ax = plt.subplots(figsize=(12, 5))
ax.plot(rates0, expected, linestyle='dotted', marker=',', color='g', label='expected')
ax.plot(rates1, c1, linestyle='solid', marker='s', color='red', label='v1 c=1')
ax.plot(rates2, c2, linestyle='solid', marker='s', color='salmon', label='v1 c=2')
ax.plot(rates4, c4, linestyle='solid', marker='s', color='orange', label='v1 c=4')
ax.plot(rates1, d1, linestyle='dashed', marker='o', color='blue', label='v0.38 c=1')
ax.plot(rates2, d2, linestyle='dashed', marker='o', color='violet', label='v0.38 c=2')
ax.plot(rates4, d4, linestyle='dashed', marker='o', color='purple', label='v0.38 c=4')

plt.title('finding the saturation point')
plt.xlabel("total rate over all connections (txs/s)")
plt.ylabel("txs processed in 90 seconds")
plt.xticks(np.arange(0, 3600, 200).tolist()) 
ax.set_xlim([0, 3600])
ax.set_ylim([0, 60000])
ax.grid()
ax.legend()

fig.tight_layout()
fig.savefig("saturation_v1_v038.png")
plt.show()
