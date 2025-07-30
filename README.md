# Display Serial Controller

A small library and cli application for controlling displays that provide a RS-232 interface.

## Usage

```text
Usage: display-serial-controller [OPTIONS] --display-type <DISPLAY_TYPE> --command <COMMAND> --value <VALUE>

Options:
  -p, --port <PORT>                  [default: /dev/ttyUSB0]
  -b, --baud-rate <BAUD_RATE>        [default: 9600]
  -m, --monitor-id <MONITOR_ID>      [default: 0]
  -d, --display-type <DISPLAY_TYPE>
  -c, --command <COMMAND>
  -v, --value <VALUE>
  -h, --help                         Print help
  -V, --version                      Print version
```

## Work in Progress

This project is still under development and not yet ready for production use.

## Supported Devices/Manufacturers

- iiyama (WIP)

## Disclaimer

This project is not affiliated with any of the manufacturers listed above. It is an independent implementation based on publicly available documentation. Use at your own risk. The author is not responsible for any damage or issues that may arise from using this software. Trademarks and brand names are the property of their respective owners.

## License

This project is licensed under the Mozilla Public License 2.0 (MPL-2.0). See the [LICENSE.txt](LICENSE.txt) file for details.
