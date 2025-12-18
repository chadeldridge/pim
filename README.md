# pim
Manage groups of Prometheus job targets with labels and jobs.

## Command
```
Export PIM data

Usage: pim <SOURCE> [TARGET]

Arguments:
  <SOURCE>  Input file path
  [TARGET]  

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### \<SOURCE>
Source can be STDIN, file, or a directory. If source is a directory, pim will read all files in the directory as source files.

### [TARGET]
If target is not specificied pim will use STDOUT. If target is a file, pim will write all output to that single file. If target is a directory, pim will write individual files per job. Providing a directory as the target is the prefered use.

If you are outputing to another command with STDOUT pim will not pretty print the data.

## Source File
```
- jobs:
    - blackbox_icmp
    - blackbox_ssh
    - node_exporter
  labels:
    environment: dev
    role: util
    application: util
  targets:
    - server1.example.com
    - server2
- jobs:
    - blackbox_ssh
  labels:
    environment: prd
    role: web-server
    application: radsite
  targets:
    - server1.example.com
    - server2
```

## Target Files
File: {dst_dir}/{job}_targets.json

File: path/to/blackbox_ssh_targets.json
```
[
  {
    "jobs": [
      "blackbox_ssh"
    ],
    "labels": {
      "application": "util",
      "environment": "dev",
      "role": "util"
    },
    "targets": [
      "server1.example.com",
      "server2"
    ]
  },
  {
    "jobs": [
      "blackbox_ssh"
    ],
    "labels": {
      "application": "radsite",
      "environment": "prd",
      "role": "webserver"
    },
    "targets": [
      "server1.example.com",
      "server2"
    ]
  }
]
```
