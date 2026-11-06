<p align="center">
  <img src="ghostline.png" alt="Ghostline route mark" width="112">
</p>

<h1 align="center">Ghostline</h1>

<p align="center">
  A native Linux desktop client for CyberGhost network locations and imported OpenVPN or WireGuard profiles.
</p>

<p align="center">
  <img alt="Rust 1.81+" src="https://emblem.xdan.cc/badge.svg?label=rust&amp;message=1.81%2B&amp;look=connected&amp;labelColor=%233D3D3D&amp;color=%23E9B44C">
  <img alt="Platform: Linux" src="https://emblem.xdan.cc/badge.svg?label=platform&amp;message=Linux&amp;look=connected&amp;labelColor=%233D3D3D&amp;color=%23E9B44C">
  <img alt="License: MIT" src="https://emblem.xdan.cc/badge.svg?label=license&amp;message=MIT&amp;look=connected&amp;labelColor=%233D3D3D&amp;color=%23E9B44C">
</p>

> Ghostline is an independent project. It is not affiliated with CyberGhost or its owner.

Ghostline uses the official `cyberghostvpn` command for live countries, cities, streaming services, servers, connection status, connect, and stop. Imported profile files continue to run through NetworkManager. Account setup and sign-in stay in the official CLI. Ghostline does not ask for your CyberGhost password and has no telemetry, analytics, ads, or upsells.

## Why Ghostline

- **Live catalogue, no invented data.** Every live country, city, service, server instance, and load value in Network comes from the CLI. When the CLI cannot answer, a clearly labelled reference country list can keep navigation usable, but it never gains cities, servers, services, load values, or a connect action.
- **One clear action per screen.** The interface is built as a quiet network tool, not a promotional dashboard. There are no banners, countdown offers, or upgrade prompts.
- **Two connection paths.** CyberGhost network locations run through the official CLI. Imported `.ovpn` and `.conf` profiles are managed by NetworkManager.
- **Honest states.** Errors name the failing component. Fallback data is labelled. A tunnel opened outside Ghostline is detected and left alone when Ghostline exits.

## How it works

| Path | Catalogue and state | Connect and stop |
| --- | --- | --- |
| CyberGhost network location | Official `cyberghostvpn` CLI | `sudo cyberghostvpn ...` through the CLI |
| Imported profile (`.ovpn`, `.conf`) | `nmcli` plus local profile parsing | `nmcli connection up` and `nmcli connection down` |

Ghostline never reads or reuses the CLI's account token. The CLI owns its session and network behavior. Ghostline runs list, status, connect, and stop commands and parses their output.

## Features

### Network

- Load live Traffic, Torrent, and Streaming catalogues from the official CLI.
- Browse countries, cities, server instances, and per-server load values.
- Choose a streaming service for the selected country.
- Search countries, cities, streaming services, and servers.
- Let the CLI choose any city or any server, or select a specific instance.
- Choose OpenVPN with UDP or TCP, or WireGuard.
- Save the selection to Locations, then connect from the summary.
- Fall back to a clearly labelled reference country list when list loading fails. City and server lists stay empty and connect actions stay disabled.

### Locations

- Keep saved CyberGhost locations next to imported `.ovpn` and `.conf` profiles.
- Search, group by country, favorite, copy an endpoint, reconnect, or remove a location.
- Show the selected protocol, server type, streaming service, load, and source.
- Import a profile through NetworkManager with optional keyring storage for its password.
- Recreate a NetworkManager connection from an imported profile.

### Connect

- Show live tunnel state, interface, transfer counters, and session information.
- Detect a tunnel already active outside Ghostline and leave it alone when Ghostline exits.
- Ask before stopping an external tunnel through the CLI.
- Check the exit IP against `api.ipify.org` only when you press the button.
- Measure latency only when you press the button.

### Activity and settings

- Local activity log with filters, search, copy, retention, and clear.
- Light, dark, and system themes.
- Launch at login, auto-connect, auto-reconnect, and optional disconnect on quit.
- Defaults for new network selections: server type, VPN protocol, and OpenVPN transport.
- Optional in-memory sudo password caching for the current run.
- Clearly labelled demo mode that does not call `nmcli`, `cyberghostvpn`, or touch real network state.

### Keyboard

| Shortcut | Action |
| --- | --- |
| `Ctrl+1` ... `Ctrl+5` | Connect, Network, Locations, Activity, Settings |
| `Ctrl+O` | Import a profile |
| `Ctrl+Enter` | Connect or disconnect |
| `Esc` | Close the open modal |

## Requirements

- Linux with an X11 or Wayland desktop and working OpenGL support.
- The official CyberGhost CLI installed and signed in for CyberGhost network locations. Setup normally starts with `sudo cyberghostvpn --configure`.
- `sudo` access for CyberGhost connect and stop operations that require it. Ghostline prompts locally and passes the password over stdin.
- NetworkManager, plus the OpenVPN and WireGuard plugins for imported profiles.
- `secret-tool` from `libsecret` if you want to save imported profile passwords in the system keyring.
- Rust 1.81 or newer to build from source.

Imported profiles are unavailable when `nmcli` is missing. CyberGhost network locations still work through the official CLI. Ghostline has no service or daemon of its own.

## Build from source

Clone or download this repository, then:

```sh
cargo build --release
./target/release/ghostline
```

To install the binary into `~/.cargo/bin`:

```sh
cargo install --path .
```

For development:

```sh
cargo test
cargo run
```

## Demo mode

Demo mode uses sample profiles, a sample catalogue, and simulated counters. It is visibly marked and does not call `nmcli` or `cyberghostvpn`, read real NetworkManager state, or change real network settings.

```sh
GHOSTLINE_DEMO=1 cargo run
```

Useful demo hooks:

```sh
GHOSTLINE_DEMO=1 GHOSTLINE_DEMO_THEME=dark cargo run
GHOSTLINE_DEMO=1 GHOSTLINE_DEMO_THEME=light cargo run
GHOSTLINE_DEMO=1 GHOSTLINE_DEMO_AUTOCONNECT=1 cargo run
GHOSTLINE_DEMO=1 GHOSTLINE_DEMO_SCREEN=network cargo run
```

`GHOSTLINE_DEMO_SCREEN` accepts `connect`, `network`, `locations`, `activity`, or `settings`.

## Account and CLI boundary

Ghostline never asks for a CyberGhost account or password. Sign in with the official client:

```sh
sudo cyberghostvpn --configure
```

The CLI owns its account session and token. If the CLI is missing or its catalogue request fails, Ghostline says so and does not substitute invented servers or availability. The bundled reference country list can keep country navigation usable, but it never gains cities, servers, services, load values, or a connect action.

CyberGhost connect and stop commands use `sudo -S --` when a password is required. The password is written to sudo over stdin, never into process arguments or Ghostline's state file. If you enable password memory, it stays in this process only and disappears on exit.

## Privacy and local data

Ghostline has no telemetry, analytics, crash reporting, ads, upsells, or remote feature flags. It does not check for updates.

Opening Network asks the official CLI for live catalogue tables. Connecting or disconnecting a CyberGhost location runs the CLI, which contacts CyberGhost using the account session already stored by the CLI. The only direct HTTP request Ghostline makes on its own is the optional exit IP check after a button press. The optional latency check opens a TCP connection to the selected imported profile.

Saved CyberGhost locations contain the country, city, streaming service, server instance, protocol, and load reported by the CLI. They do not contain account credentials. The sudo password is never written to disk. Imported profile passwords use the system keyring when saved, or a private temporary NetworkManager credentials file that is deleted after the attempt.

Ghostline keeps its state and logs under the XDG directories:

- `$XDG_CONFIG_HOME/ghostline/state.json`
- `$XDG_DATA_HOME/ghostline/logs/`

See [docs/privacy.md](docs/privacy.md) for the full data map, file permissions, and removal steps.

## Project status

Version 0.1.0. The live CLI catalogue and external tunnel detection were verified against `cyberghostvpn` 1.5.0. The CyberGhost connection path is implemented and unit tested, but a new tunnel was not opened from Ghostline while an external tunnel was active on the test machine. Imported profile connections still need validation with real credentials.

## Limitations

- No CyberGhost account login inside Ghostline. Setup stays in the official CLI.
- No kill switch in this version.
- No tray icon or hide-to-tray behavior yet.
- No map. A map without real coordinates would be decoration.
- No Windows or macOS build.
- A new CyberGhost tunnel from Ghostline was not opened during testing while an external tunnel was active. Status detection, catalogue access, profile saving, and exit behavior were verified without disturbing it.
- Imported profile tunnels remain unverified with real CyberGhost credentials.

## Documentation

- [Privacy and data map](docs/privacy.md)
- [Interoperability and reverse-engineering notes](docs/reverse-engineering.md)
- [Design direction](DESIGN.md)
- [Third-party notices](THIRD_PARTY_NOTICES.md)

## Contributing

Bug reports and pull requests are welcome. Please include your distribution, desktop session (X11 or Wayland), `cyberghostvpn --version` when relevant, and whether the issue involves a CyberGhost location or an imported profile.

## License

Ghostline source code is released under the MIT license. See [LICENSE](LICENSE) and [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

Bundled IBM Plex fonts are under the SIL Open Font License, with the license text in `assets/fonts/OFL.txt`. The fallback country table is derived from the MIT-licensed `cyberghostvpn-gui` project. The proprietary CyberGhost installer is a local research input. It is excluded in `.gitignore` and must not be redistributed with Ghostline.

CyberGhost and its logos are trademarks of their respective owner. Ghostline is an independent project and is not affiliated with CyberGhost.
