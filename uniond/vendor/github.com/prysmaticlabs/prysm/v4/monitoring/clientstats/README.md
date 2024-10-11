# Client stats reporting

## Specification 

The request JSON object is a non-nested object with the following properties. The process refers to which process data is associated with the request.


|Property                           |Type         |Process              |Source                                                                   |Description                                                                                                                                                             |
|-----------------------------------|-------------|---------------------|-------------------------------------------------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
|version                            |int          |Every request        |hard-coded                                                               |Stats data specification version, current only `1` is accepted                                                                                                          |
|timestamp                          |long         |Every request        |system time                                                              |Unix timestamp in milliseconds                                                                                                                                          |
|process                            |string (enum)|Every request        |hard-coded / cli flag                                                    |Enum values: validator, beaconnode, system                                                                                                                              |
|cpu_process_seconds_total          |long         |beaconnode, validator|prom: process_cpu_seconds_total                                          |CPU seconds consumed by the process                                                                                                                                     |
|memory_process_bytes               |long         |beaconnode, validator|prom: process_resident_memory_bytes                                      |Number of bytes allocated to the process                                                                                                                                |
|client_name                        |string       |beaconnode, validator|hard-coded to "prysm"                                                    |Name of the client type. Ex: prysm, lighthouse, nimbus, teku                                                                                                            |
|client_version                     |string       |beaconnode, validator|prom: prysm_version (label: version)                                     |Client version. Ex: 1.0.0-beta.0                                                                                                                                        |
|client_build                       |int          |beaconnode, validator|prom: prysm_version (label: buildDate)                                   |Integer representation of build for easier comparison                                                                                                                   |
|disk_beaconchain_bytes_total       |long         |beaconchain          |prom: bcnode_disk_beaconchain_bytes_total                                |The amount of data consumed on disk by the beacon chain's database.                                                                                                     |
|network_libp2p_bytes_total_receive |long         |beaconchain          |(currently unsupported)                                                  |The number of bytes received via libp2p traffic                                                                                                                         |
|network_libp2p_bytes_total_transmit|long         |beaconchain          |(currently unsupported)                                                  |The number of bytes transmitted via libp2p traffic                                                                                                                      |
|network_peers_connected            |int          |beaconchain          |(currently unsupported)                                                  |The number of peers currently connected to the beacon chain                                                                                                             |
|sync_eth1_connected                |bool         |beaconchain          |prom: powchain_sync_eth1_connected                                       |Whether or not the beacon chain node is connected to a _synced_ eth1 node                                                                                               |
|sync_eth2_synced                   |bool         |beaconchain          |prom: beacon_clock_time_slot (true if this equals prom: beacon_head_slot)|Whether or not the beacon chain node is in sync with the beacon chain network                                                                                           |
|sync_beacon_head_slot              |long         |beaconchain          |prom: beacon_head_slot                                                   |The head slot number.                                                                                                                                                   |
|sync_eth1_fallback_configured      |bool         |beaconchain          |prom: powchain_sync_eth1_fallback_configured                             |Whether or not the beacon chain node has a fallback eth1 endpoint configured.                                                                                           |
|sync_eth1_fallback_connected       |bool         |beaconchain          |prom: powchain_sync_eth1_fallback_connected                              |Whether or not the beacon chain node is connected to a fallback eth1 endpoint. A true value indicates a failed or interrupted connection with the primary eth1 endpoint.|
|slasher_active                     |bool         |beaconchain          |(coming soon)                                                            |Whether or not slasher functionality is enabled.                                                                                                                        |
|sync_eth2_fallback_configured      |bool         |validator            |(currently unsupported)                                                  |Whether or not the process has a fallback eth2 endpoint configured                                                                                                      |
|sync_eth2_fallback_connected       |bool         |validator            |(currently unsupported)                                                  |Weather or not the process has connected to the failover eth2 endpoint. A true value indicates a failed or interrupted connection with the primary eth2 endpoint.       |
|validator_total                    |int          |validator            |prom: validator_statuses (count of all peers)                            |The number of validating keys in use.                                                                                                                                   |
|validator_active                   |int          |validator            |prom: validator_statuses (count of peers w/ "ACTIVE" status label)       |The number of validator keys that are currently active.                                                                                                                 |
|cpu_cores                          |int          |system               |(currently unsupported)                                                  |The number of CPU cores available on the host machine                                                                                                                   |
|cpu_threads                        |int          |system               |(currently unsupported)                                                  |The number of CPU threads available on the host machine                                                                                                                 |
|cpu_node_system_seconds_total      |long         |system               |(currently unsupported)                                                  |Overall CPU seconds observed on the host machine for all processes.                                                                                                     |
|cpu_node_user_seconds_total        |long         |system               |(currently unsupported)                                                  |??                                                                                                                                                                      |
|cpu_node_iowait_seconds_total      |long         |system               |(currently unsupported)                                                  |??                                                                                                                                                                      |
|cpu_node_idle_seconds_total        |long         |system               |(currently unsupported)                                                  |??                                                                                                                                                                      |
|memory_node_bytes_total            |long         |system               |(currently unsupported)                                                  |??                                                                                                                                                                      |
|memory_node_bytes_free             |long         |system               |(currently unsupported)                                                  |??                                                                                                                                                                      |
|memory_node_bytes_cached           |long         |system               |(currently unsupported)                                                  |??                                                                                                                                                                      |
|memory_node_bytes_bufferd          |long         |system               |(currently unsupported)                                                  |??                                                                                                                                                                      |
|disk_node_bytes_total              |long         |system               |(currently unsupported)                                                  |??                                                                                                                                                                      |
|disk_node_bytes_free               |long         |system               |(currently unsupported)                                                  |??                                                                                                                                                                      |
|disk_node_io_seconds               |long         |system               |(currently unsupported)                                                  |??                                                                                                                                                                      |
|disk_node_reads_total              |long         |system               |(currently unsupported)                                                  |??                                                                                                                                                                      |
|disk_node_writes_total             |long         |system               |(currently unsupported)                                                  |??                                                                                                                                                                      |
|network_node_bytes_total_receive   |long         |system               |(currently unsupported)                                                  |??                                                                                                                                                                      |
|network_node_bytes_total_transmit  |long         |system               |(currently unsupported)                                                  |??                                                                                                                                                                      |
|misc_node_boot_ts_system           |long         |system               |(currently unsupported)                                                  |??                                                                                                                                                                      |
|misc_os                            |string       |system               |(currently unsupported)                                                  |Enum values: lin, win, mac, unk                                                                                                                                         |

The client stats reporter will submit a request object for each process type. The report request may 
submit a list of data or a single JSON object.

### Examples

POST https://beaconcha.in/api/v1/stats/$API_KEY/$MACHINE_NAME

**Single object payload**

```json
{
   "version": 1,
   "timestamp": 11234567,
   "process": "validator",
   "cpu_process_seconds_total": 1234567,
   "memory_process_bytes": 654321,
   "client_name": "lighthouse",
   "client_version": "1.1.2",
   "client_build": 12,
   "sync_eth2_fallback_configured": false,
   "sync_eth2_fallback_connected": false,
   "validator_total": 3,
   "validator_active": 2
}
```

**Multiple object payload**

```json
[
   {
  	"version":1,
  	"timestamp":1618835497239,
  	"process":"beaconnode",
  	"cpu_process_seconds_total":6925,
  	"memory_process_bytes":1175138304,
  	"client_name":"lighthouse",
  	"client_version":"1.1.3",
  	"client_build":42,
  	"sync_eth2_fallback_configured":false,
  	"sync_eth2_fallback_connected":false,
  	"validator_active":1,
  	"validator_total":1
   },
   {
  	"version":1,
  	"timestamp":1618835497258,
  	"process":"system",
  	"cpu_cores":4,
  	"cpu_threads":8,
  	"cpu_node_system_seconds_total":1953818,
  	"cpu_node_user_seconds_total":229215,
  	"cpu_node_iowait_seconds_total":3761,
  	"cpu_node_idle_seconds_total":1688619,
  	"memory_node_bytes_total":33237434368,
  	"memory_node_bytes_free":500150272,
  	"memory_node_bytes_cached":13904945152,
  	"memory_node_bytes_buffers":517832704,
  	"disk_node_bytes_total":250436972544,
  	"disk_node_bytes_free":124707479552,
  	"disk_node_io_seconds":0,
  	"disk_node_reads_total":3362272,
  	"disk_node_writes_total":47766864,
  	"network_node_bytes_total_receive":26546324572,
  	"network_node_bytes_total_transmit":12057786467,
  	"misc_node_boot_ts_seconds":1617707420,
  	"misc_os":"unk"
   }
]
```
