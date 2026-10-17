# waybar-performance-monitor

Waybar custom module that reads Linux CPU Pressure Stall Information (PSI) from `/proc/pressure/cpu` and outputs the 10-second average (`avg10`) with Waybar alert classes.

## Build and Installation

```shell
cargo build --release
mkdir -p ~/.config/waybar/scripts
cp target/release/waybar-performance-monitor ~/.config/waybar/scripts/waybar-performance-monitor
```

## Usage

`~/.config/waybar/config`:

```json
"custom/performance_monitor": {
    "exec": "~/.config/waybar/scripts/waybar-performance-monitor",
    "return-type": "json"
}
```

## Alert Classes

| `avg10` (rounded) | CSS Class |
| ----------------- | --------- |
| `< 5%`            | None      |
| `5% - 9%`         | `info`    |
| `10% - 20%`       | `warning` |
| `> 20%`           | `critical`|
