# hidctl

One-shot CLI for listing HID devices, selecting a device, and sending or receiving raw HID and feature reports.

## Build

```sh
cargo build --release
./target/release/hidctl list
```

## Commands

List all devices, optionally filtering by VID/PID:

```sh
hidctl list
hidctl list --json
hidctl list --vid 0x1234 --pid 0x5678
```

Show one device. Report commands use the same selector options. A selector must identify exactly one device; a first match is never chosen implicitly.

```sh
hidctl info --vid 0x1234 --pid 0x5678 --usage-page 0x0001 --usage 0x0002
hidctl info --path '<device-path>'
hidctl info --serial '<serial-number>'
```

Send and receive reports:

```sh
hidctl send-output \
  --vid 0x1234 --pid 0x5678 --usage-page 0x0001 --usage 0x0002 \
  --bytes '0x01,0x02,0x03'

hidctl read-input \
  --vid 0x1234 --pid 0x5678 --usage-page 0x0001 --usage 0x0002 \
  --timeout 500

hidctl send-feature --path '<device-path>' --bytes '0x11,0x01,0x02'
hidctl read-feature --path '<device-path>' --report-id 0x11 --length 64
```

`--bytes` accepts comma or whitespace separators. `0x` values are always hexadecimal. A list containing `a`-`f` is interpreted as hexadecimal, while a digits-only list is interpreted as decimal. `--length` requires an exact length for send commands.

## Exit Codes

```text
0  success
1  HIDAPI initialization or OS error
2  argument or byte parsing error
3  no matching device
4  multiple matching devices
5  report I/O error
```
