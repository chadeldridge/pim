# pim
Manage groups of Prometheus job targets with labels and jobs.

## Command
```
Usage: pim [OPTIONS]

Options:
  -s, --source <SOURCE>  Input source file path. Can be a file or directory
  -t, --target <TARGET>  Output target file path. Can be a file or directory
  -h, --help             Print help
  -V, --version          Print version
```

### Common Usage
Write output to stdout.

`pim -s testdata/test.yml`

Write target files to directory.

`pim -s testdata/test.yml -t testdata/targets/`

### Source
Source can be STDIN, file, or a directory. If source is a directory, pim will read all files in the directory as source files.

### Target
If target is not specified pim will use STDOUT. If target is a file, pim will write all output to that single file. If target is a directory, pim will write individual files per job. Providing a directory as the target is the preferred use.

If you are outputting to another command with STDOUT pim will not pretty print the data.

## Source File
Formatting of the source file should follow.
```
- jobs:
  labels:
  targets:
```
 - jobs: A list of jobs to run against all targets.
 - labels: Optional map of labels to be added to all targets.
 - targets: List of targets for this group.

Targets should be grouped together based on the jobs that should be ran against all targets in the group and labels to be applied to all targets in the group.

### Example
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

# Installation
Download the archive for your OS from the release page and place the binary in your path.
```
$ echo $PATH | tr ':' '\n'
/home/user/.local/bin
/home/user/bin
/usr/local/sbin
/usr/local/bin
/usr/sbin
/usr/bin
/sbin
/bin
/usr/games
/usr/local/games
/snap/bin
$ tar xfz pim-v0.1.0-x86_64-unknown-linux-gnu.tar.gz 
$ cp pim-v0.1.0-x86_64-unknown-linux-gnu/pim ~/bin/
$ pim --version
pim 0.1.0
```

A windows EXE is also available.