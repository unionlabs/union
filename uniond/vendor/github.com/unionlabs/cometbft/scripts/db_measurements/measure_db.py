#!/usr/bin/env python3

import subprocess
import asyncio
import os
import signal


LOCAL_NET_LOG = os.path.join("scripts", "db_measurements", "local_net.log")
PAYLOAD_LOG = os.path.join("scripts", "db_measurements", "payload.log")


def set_project_root():
	current_file_path = os.path.abspath(__file__)
	candidate = os.path.dirname(os.path.dirname(os.path.dirname(current_file_path)))
	if os.path.basename(candidate) == "cometbft":
		os.chdir(candidate)
		return
	raise FileNotFoundError("Can't locate project root")


async def read_and_handle_process_output(process, handle, *handle_args):
	while True:
		line = await process.stdout.readline()
		if not line:
			break
		ln = line.decode().strip()
		handle(ln, *handle_args)
		


async def clear_n_launch_localnet(node_started_event):
	clear_command = ["sudo", "rm", "-rf", "build/"]
	build_command = ["sudo", "make", "build-linux"]
	launch_command = ["sudo", "make", "localnet-start"]
	subprocess.run(clear_command, capture_output=True, text=True, check=True)
	subprocess.run(build_command, capture_output=True, text=True, check=True)
	process = await asyncio.create_subprocess_exec(*launch_command, stdout=asyncio.subprocess.PIPE, stderr=asyncio.subprocess.PIPE)

	def handle(line, event):
		if "Started node" in line:
			event.set()
		with open(LOCAL_NET_LOG, 'a') as f:
			f.write(line + "\n")

	asyncio.create_task(read_and_handle_process_output(process, handle, node_started_event))


def prettify_du(s):
	return '; '.join([f"{os.path.basename(l.split('	')[1])}: {l.split('	')[0]}" for l in filter(len, s.split('\n'))])
		

async def measure_du(rate, tx_size):
	data_path = os.path.join("build", "node0", "data")
	measure_command = ["sudo", "du", "-h", data_path]
	T = 0
	period = 20
	try:
		while True:
			try:
				result = subprocess.run(measure_command, capture_output=True, text=True, check=True, timeout=10)
				pretty_result = prettify_du(result.stdout)
				print(f"r {rate}; s {tx_size}; T {T}; storage size: {pretty_result}")
			except:
				pass
			await asyncio.sleep(period)
			T += period
	except asyncio.CancelledError:
		print() 


async def launch_payload(r, s, T):
	payload_script_path = os.path.join("test", "loadtime", "build", "load")
	payload_command = [payload_script_path, "-T", str(T), "-r", str(r), "-s", str(s),  "--endpoints", "ws://localhost:26657/websocket"]
	process = await asyncio.create_subprocess_exec(*payload_command, stdout=asyncio.subprocess.PIPE, stderr=asyncio.subprocess.PIPE)

	def handle(line, *args):
		print("Payloading")
		with open(PAYLOAD_LOG, 'a') as f:
			f.write(line + "\n")

	asyncio.create_task(read_and_handle_process_output(process, handle))
	await process.wait()


def stop_localnet():
	stop_command = ["sudo", "make", "build", "localnet-stop"]
	subprocess.run(stop_command, capture_output=True, text=True, check=True)


async def run():
	tx_sizes = [4096]
	rates = [1500]
	T = 3600
	for tx_size in tx_sizes:
		for rate in rates:
			node_started_event = asyncio.Event()
			localnet_process = asyncio.create_task(clear_n_launch_localnet(node_started_event))
			await node_started_event.wait()
			payload_task = asyncio.create_task(launch_payload(rate, tx_size, T))
			measure_task = asyncio.create_task(measure_du(rate, tx_size))
			await payload_task
			measure_task.cancel()
			stop_localnet()


def main():
	set_project_root()
	with open(LOCAL_NET_LOG, 'w'):
		pass
	with open(PAYLOAD_LOG, 'w'):
		pass
	asyncio.run(run())


main()