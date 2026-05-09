# ifs

Rust replacement for `~/scripts/ifs`.

## Discovery

`ifs` is a lightweight Linux network interface monitor written in Rust. It
lists network interfaces and IPv4 addresses in one-shot mode, and provides a
terminal UI for real-time Rx/Tx bandwidth monitoring in interval mode.

Useful search terms: Linux network interface monitor, terminal UI network
bandwidth monitor, Rust networking CLI, interface Rx/Tx rates, sysfs network
statistics, ratatui network monitor, deb package, rpm package.

## Build

```sh
make build
```

## Install

```sh
sudo make install
```

The default install path is `/usr/local/bin/ifs`. Override it with `PREFIX`,
`BINDIR`, or `INSTALL_PATH`:

```sh
make build
sudo make install PREFIX=/usr
```

## Packages

```sh
make package
make check-packages
```

Packages are written to `dist/`:

- `ifs_<version>_amd64.deb`
- `ifs-<version>-1.x86_64.rpm`

## Usage

```sh
ifs
ifs --all
ifs -h
ifs -i
ifs -i --bits
ifs -i 0.5
ifs --interval 3
```

One-shot mode prints interface names and IPv4 addresses immediately. Monitor
mode uses a terminal UI and adds Rx/Tx bandwidth columns. Rx/Tx rates default
to byte units; pass `--bits` to show network-style bit units.
