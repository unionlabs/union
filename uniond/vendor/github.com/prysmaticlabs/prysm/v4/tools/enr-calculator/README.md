# ENR Calculator

To generate the ENR of a node

```
 bazel run //tools/enr-calculator:enr-calculator -- --private CAISIJXSWjkbgprwuo01QCRegULoNIOZ0yTl1fLz5N0SsJCS --ipAddress 127.0.0.1 --port 2000

```

This will deterministically generate an ENR from given inputs of private key, ip address and udp port.

Output of the above command:
```
INFO[0000] enr:-IS4QKk3gX9EqxA3x83AbCiyAnSuPDMvK52dC50Hm1XGDd5tEyQhM3VcJL-4b8kDg5APz_povv0Syqk0nancoNW-cq0BgmlkgnY0gmlwhH8AAAGJc2VjcDI1NmsxoQM1E5yUsp9vDQj1tv3ZWXCvaFBvrPNdz8KPI1NhxfQWzIN1ZHCCB9A

```