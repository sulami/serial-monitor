# Serial Monitor

Born out of the desire to connect to an Arduino without having to run
the whole Arduino suite.

## Usage

```sh
$ serial-monitor -h
serial-monitor 0.1.0
Robin Schroer <git@sulami.xyz>
Simple Arduino Serial Monitor

USAGE:
    serial-monitor [OPTIONS] --path <PATH>

OPTIONS:
    -b, --baud <BAUD>    Baud rate [default: 9600]
    -h, --help           Print help information
    -p, --path <PATH>    Path of the TTY device, e.g. /dev/tty123
    -V, --version        Print version information
```

## Building

```sh
cargo build
```

Who would have thought.
