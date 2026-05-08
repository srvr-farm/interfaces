# ifs

Rust replacement for `~/scripts/ifs`.

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
ifs -i 0.5
ifs --interval 3
```

One-shot mode prints interface names and IPv4 addresses immediately. Monitor
mode uses a terminal UI and adds Rx/Tx bandwidth columns.
