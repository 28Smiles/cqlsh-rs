[![Main](https://github.com/28Smiles/cqlsh-rs/actions/workflows/test.yml/badge.svg)](https://github.com/28Smiles/cqlsh-rs/actions/workflows/test.yml)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)
[![Latest Stable](https://img.shields.io/github/v/release/28Smiles/cqlsh-rs?label=latest%20stable)](https://github.com/28Smiles/cqlsh-rs/releases/latest)
[![Latest Release](https://img.shields.io/github/v/release/28Smiles/cqlsh-rs?include_prereleases&label=latest%20release)](https://github.com/28Smiles/cqlsh-rs/releases)

# cqlsh-rs: The Rust-CQL shell

cqlsh-rs is a command-line interface for interacting with Cassandra using CQL (the Cassandra Query Language). Originally shipped with every Cassandra package and implemented in Python 2, this tool is reimplemented in Rust.

## Command Line Options

Usage:

`cqlsh [options] [host [port]]`

Options:

`-u --user`

Username to authenticate against Cassandra with

`-p --password`

Password to authenticate against Cassandra with, should be used in conjunction with --user

`-k --keyspace`

Keyspace to authenticate to, should be used in conjunction with --user

`-f --file`

Execute commands from the given file, then exit

`-e --execute`

Execute the given statement, then exit

`--connect-timeout`

Specify the connection timeout in seconds (defaults to 2s)

## Special Commands
In addition to supporting regular CQL statements, cqlsh also supports a number of special commands that are not part of CQL. These are detailed below.

`EXIT`

Ends the current session and terminates the cqlsh process.
