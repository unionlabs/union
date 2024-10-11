#!/usr/bin/python3
#
# Modified version of latsetter.py script from 
# https://github.com/paulo-coelho/latency-setter

import csv
import os
import sys
import netifaces as nif


def usage():
    print("Usage:")
    print(f"\t{sys.argv[0]} set ip-zones-csv latency-matrix-csv interface-name")
    print("or")
    print(f"\t{sys.argv[0]} unset interface-name")
    exit(1)


def unset_latency(iface):
    os.system(f"tc qdisc del dev {iface} root")


def read_ip_zones_file(path):
    """ The first line of the file should contain the column names, corresponding to: node-name, ip,
    and zone. """
    with open(path, "r") as ips:
        return list(csv.DictReader(ips))


def read_latency_matrix_file(path):
    with open(path, "r") as f:
        lines = list(list(rec) for rec in csv.reader(f, delimiter=","))
        all_zones = lines[0][1:] # discard first element of the first line (value "from/to")
        zone_latencies = lines[1:] # each line is a zone name, followed by the latencies to other nodes

        # Convert lines of comma-separated values to dictionary from pairs of zones to latencies.
        latencies = {}
        for ls in zone_latencies:
            source_zone = ls[0]
            for i, dest_zone in enumerate(all_zones):
                value = float(ls[i+1])
                if value != 0:
                    latencies[(source_zone,dest_zone)] = value
        
        return all_zones, latencies


def set_latency(ips_path, latencies_path, iface):
    ip_zones = read_ip_zones_file(ips_path)
    all_zones, latencies = read_latency_matrix_file(latencies_path)

    # Get my IP address
    try:
        myip = nif.ifaddresses(iface)[nif.AF_INET][0]["addr"]
    except ValueError:
        print(f"IP address not found for {iface}")
        exit(1)

    # Get my zone and node name
    myzone, mynode = next(((e["Zone"], e["Node"]) for e in ip_zones if e["IP"] == myip), (None, None))
    if not myzone:
        print(f"No zone configured for node {myip}")
        exit(1)

    # print(f"# Setting rules for interface {iface} in zone {myzone} with IP {myip}")

    # TODO: explain the following commands
    os.system(f"tc qdisc del dev {iface} root 2> /dev/null")
    os.system(f"tc qdisc add dev {iface} root handle 1: htb default 10")
    os.system(f"tc class add dev {iface} parent 1: classid 1:1 htb rate 1gbit 2> /dev/null")
    os.system(f"tc class add dev {iface} parent 1:1 classid 1:10 htb rate 1gbit 2> /dev/null")
    os.system(f"tc qdisc add dev {iface} parent 1:10 handle 10: sfq perturb 10")

    handle = 11 # why this initial number?
    for zone in all_zones:
        lat = latencies.get((myzone,zone))
        if not lat or lat <= 0:
            continue
        
        delta = .05 * lat
        
        # TODO: explain the following commands
        os.system(f"tc class add dev {iface} parent 1:1 classid 1:{handle} htb rate 1gbit 2> /dev/null")
        os.system(f"tc qdisc add dev {iface} parent 1:{handle} handle {handle}: netem delay {lat:.2f}ms {delta:.2f}ms distribution normal")

        for item in ip_zones:
            if item["Zone"] == zone:
                ip = item["IP"]
                node = item["Node"]
                print(f"# Setting latency from {mynode}/{myip} ({myzone}) to {node}/{ip} ({zone}): {lat:.2f}ms +/- {delta:.2f}ms")
                os.system(f"tc filter add dev {iface} protocol ip parent 1: prio 1 u32 match ip dst {ip}/32 flowid 1:{handle}")
        
        handle += 1


if __name__ == "__main__":
    if len(sys.argv) < 3:
        usage()

    if sys.argv[1] == "unset" and sys.argv[2]:
        unset_latency(sys.argv[2])
    elif sys.argv[1] == "set" and sys.argv[2] and sys.argv[3] and sys.argv[4]:
        set_latency(sys.argv[2], sys.argv[3], sys.argv[4])
    else:
        usage()
