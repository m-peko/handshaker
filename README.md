[![Build](https://github.com/m-peko/handshaker/actions/workflows/ci.yml/badge.svg)](https://github.com/m-peko/handshaker/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT-blue)](https://opensource.org/licenses/mit)

handshaker - Perform P2P handshake with Bitcoin node
====================================================

Table of Content
----------------

1. [Quick Start](#quick-start)
2. [Contribution](#contribution)
3. [License](#license)

Quick Start
-----------

Prerequisites:

- Rust toolchain
- Docker (optional)

Perform P2P handshake to one or more Bitcoin nodes by running the following command:

```bash
cargo run 75.30.104.234:8333 185.78.209.28:8333
```

In case you want to run your local Bitcoin node, follow the next steps:

1. Build Docker image

```bash
docker build docker/ -t bitcoin-node
```

2. Run Docker container

```bash
docker run -p 18444:18444 --name btc bitcoin-node
```

Port 18444 is used in the `regtest` Bitcoin network by default.

If you wish to run the node in Bitcoin network other than `regtest` (possible values:
`main`, `test`, `signet`, `regtest`), set the environment variable like:

```bash
docker run -e BTC_CHAIN=main ...
```

3. Perform P2P handshake

```bash
cargo run 0.0.0.0:18444 -n testnet
```

_Note: Checksum check might fail sometimes during handshake._

Contribution
------------

Feel free to contribute.

If you find that any of the tests **fail**, please create a ticket in the issue tracker indicating the following information:

* platform
* architecture

License
-------

The project is available under the [MIT](https://opensource.org/licenses/MIT) license.
