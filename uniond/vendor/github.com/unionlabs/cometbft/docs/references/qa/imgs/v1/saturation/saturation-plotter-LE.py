#/usr/bin/env python3
"""Plotter for comparing saturation results on v1 with and without Letency Emulation.

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

# Transaction rate (x axis)
rates = np.arange(100, 1100, 100)
rates2 = [200, 400, 800, 1600]

# expected values
expected = [(i+1) * 100 * 89 for i in range(10)]

# v1 without LE
d1 = [8900,17800,26053,28800,32513,30455,33077,32191,30688,32395] # experiments/2024-03-26-13_47_51N/validator174
d2 = [8900,17800,26300,25400,31371,31063,31603,32886,24521,25211] # experiments/2024-03-26-11_17_58N/validator174
d3 = [8900,17800,26700,35600,38500,40502,51962,48328,50713,42361] # experiments/2024-03-26-20_20_33N/validator174

# v1 with LE
le1 = [8900,17800,26700,35600,34504,42169,38916,38004,34332,36948] # experiments/2024-03-25-17_41_09N/validator174
le2 = [17800, 33800, 34644, 43464] # experiments/2024-03-25-12_17_12N/validator174
le3 = [8900,17800,26700,33200,37665,51771,38631,49290,51526,46902] # experiments/2024-03-26-22_21_31N/validator174

fig, ax = plt.subplots(figsize=(10, 5))
ax.plot(rates, expected, linestyle='dotted', marker=',', color='g', label='expected')
ax.plot(rates, d1, linestyle='solid', marker='o', color='b', label='without LE #1')
ax.plot(rates, d2, linestyle='solid', marker='o', color='violet', label='without LE #2')
ax.plot(rates, d3, linestyle='solid', marker='o', color='grey', label='without LE #3')
ax.plot(rates, le1, linestyle='dashed', marker='s', color='r', label='with LE #1')
ax.plot(rates2, le2, linestyle='dashed', marker='s', color='orange', label='with LE #2')
ax.plot(rates, le3, linestyle='dashed', marker='s', color='brown', label='with LE #3')

plt.title('saturation point for v1.0.0-alpha.2, c=1')
plt.xlabel("rate (txs/s)")
plt.ylabel("txs processed in 90 seconds")
plt.xticks(np.arange(0, 1100, 200).tolist()) 
ax.set_xlim([0, 1100])
ax.set_ylim([0, 60000])
ax.grid()
ax.legend()

fig.tight_layout()
fig.savefig("saturation_v1_LE.png")
plt.show()
