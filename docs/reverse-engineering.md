# Interoperability and reverse-engineering notes

This document records what was inspected while building Ghostline, what was learned, and what was deliberately not reused. It is a provenance record, not an endorsement of CyberGhost.

## Scope

Ghostline is an independent Linux client for CyberGhost network locations and imported OpenVPN or WireGuard profile files. The work had these inputs:

1. The official Windows installer, inspected to understand the product's components, telemetry posture, and profile formats.
2. The official Linux CLI `cyberghostvpn` 1.5.0, used as the catalogue, connection, and status backend.
3. The installed, MIT-licensed `cyberghostvpn-gui` 1.0.7, used as a reference for Linux command shapes and country metadata.
4. Live CLI output from an existing signed-in session, used to verify table formats and catalogue behavior.

No CyberGhost password or profile file was read or stored by Ghostline during the work. A tunnel was already active outside Ghostline while the live catalogue and exit behavior were tested, so that tunnel was left alone.

## Official Windows installer

The supplied `CyberGhostVPNSetup.exe` is a small .NET bootstrapper. It downloads a larger payload archive and installs several applications.

The extracted payload contains:

- A WPF dashboard, including WebView2-based content.
- OpenVPN binaries and Windows network driver components.
- Separate AntiVirus, PrivacyGuard, Updater, and VPN application directories.
- Service and pipe components used by the dashboard.
- Analytics, crash-reporting, experimentation, and install-attribution libraries. These include Mixpanel-related bootstrapper code, Sentry, RudderAnalytics, and LaunchDarkly SDKs. The bootstrapper also has ClickData handling for campaign and installation context.

Finding an SDK is not proof that a particular event is sent in every build or region. It is proof that the capability exists in the shipped payload. The presence of campaign and installation context code is also consistent with attribution and conversion tracking.

Ghostline implements none of these capabilities. It has no analytics, crash reporting, remote flags, campaign parameters, install counters, or update check. It does not embed the official client's dashboard or promotional surfaces.

## Installed Linux tools

The reference Linux installation had:

- `cyberghostvpn` 1.5.0, the official command-line client. The executable is a PyInstaller-packed binary at `/usr/local/cyberghost/cyberghostvpn`, linked from `/usr/bin/cyberghostvpn`.
- `cyberghostvpn-gui` 1.0.7, an unofficial MIT-licensed Fyne GUI.

The GUI stores its settings and cached country list in `~/.config/cyberghostvpn-gui/settings.conf`. It invokes the official CLI for connection work. Its settings include launch behavior, auto-connect, password memory, language, tray behavior, and disconnecting the VPN on exit.

### CLI behavior observed

The CLI accepts these catalogue shapes:

```text
cyberghostvpn --traffic --country-code
cyberghostvpn --torrent --country-code
cyberghostvpn --streaming --country-code US
cyberghostvpn --traffic --country-code DE
cyberghostvpn --traffic --country-code DE --city Berlin
cyberghostvpn --streaming "Fandango US" --country-code US
cyberghostvpn --streaming "Fandango US" --country-code US --city "New York"
```

The tables use pipe-delimited rows:

- Countries: number, country name, country code.
- Cities: number, city, instance count, users load.
- Servers: number, server name, server instance, users load.
- Streaming services: number, service name, country code.

Important details found during verification:

- The city table's `Instance` column is a count of servers in that city, not a server identifier. Ghostline stores it as `City.instances` and displays it as "N servers".
- Streaming server tables may repeat the numeric `No.` value `1` for every row. Numeric row identity is not unique. Ghostline keys server rows by instance and city rows by name.
- Streaming service names contain spaces and country suffixes, such as `Amazon Prime US`, `Fandango US`, and `Netflix US`. They are passed as one argument after `--streaming`.
- The catalogue can fail transiently with a JSON decode traceback from the CLI. Ghostline treats an empty parsed list plus an error line as a failure and does not invent rows.
- `cyberghostvpn --status` prints `VPN connection found.` when active. The inactive text contains the same words with a negative prefix, so a substring check for `VPN connection found` can produce a false positive. Ghostline checks negative forms first (`No VPN connection found`, `No VPN connections found`, `not connected`) and tests that behavior.
- The native GUI passes a UDP or TCP `--connection` value even for WireGuard. Ghostline omits `--connection` for WireGuard because WireGuard does not use that choice.

### What Ghostline reused

- Country code and country name facts from the installed GUI's cached country list. The full set present at the time of the work was synchronized into `src/countries.rs` and is used only as a navigation fallback.
- The general Linux feature checklist: launch at login, auto-connect, automatic reconnect, disconnect on quit, compact rows, and local profile management.
- The official CLI's command shapes and table formats, reimplemented in Rust.
- The observation that profile files are a practical interoperability boundary for imported NetworkManager connections.

### What Ghostline did not reuse

- The GUI's `profiles` array or any imported profile data.
- CyberGhost flags, icons, images, translations, themes, or other visual assets.
- The GUI's Go source structure as code. The Rust integration invokes the official CLI directly and parses its output.
- The official Windows application's code, resources, endpoints, campaign parameters, or telemetry integrations.
- The official CLI's proprietary binaries or bundled Python application code.

The country table is factual metadata and the GUI is MIT licensed. Attribution is included in `THIRD_PARTY_NOTICES.md`.

## Independent implementation

Ghostline is written in Rust with `eframe`/`egui`. It has two connection paths.

CyberGhost network locations:

- The official CLI lists countries, cities, streaming services, servers, and loads.
- Connect and stop are delegated to the official CLI with `sudo -S --`.
- Status is read with `cyberghostvpn --status`.
- A tunnel already active outside Ghostline is detected at startup and watched without being stopped automatically on exit.

Imported profile files:

- Import uses `nmcli connection import` for OpenVPN and WireGuard profiles.
- Connection and disconnection use `nmcli connection up` and `nmcli connection down`.
- State is reconciled against NetworkManager's active connection list and interface counters.
- Profiles are parsed locally for endpoint, protocol, authentication requirement, country, and city classification.

This keeps privileged networking work in the system service or the official CLI that already owns it, and keeps Ghostline's own behavior observable and replaceable.

## Catalogue data boundary

Live catalogue data is not bundled. Ghostline runs the official CLI on demand and holds the response in memory. Only a saved location's selection is written to Ghostline's state file.

The bundled fallback country table is used only when the CLI cannot answer before a successful list. It can support country navigation, but it never gains cities, servers, services, load values, or a connect action. Actions are disabled while the fallback is in use. The interface labels the fallback explicitly.

Ghostline does not scrape the CyberGhost website, call undocumented account APIs, or reuse a CLI token. The CLI owns that session.

## Telemetry and advertising decisions

The official client's stacked promotions, promotional banners, upsell surfaces, campaign attribution, and analytics are not part of Ghostline. The design notes in `DESIGN.md` treat those patterns as an anti-reference.

Ghostline is intentionally different in these areas:

- No account login inside the app.
- No embedded marketing, banners, or upgrade prompts.
- No kill switch.
- No traffic inspection.
- No automatic update or feature-flag request.
- No invented latency. A latency value is shown only after a user-triggered TCP measurement, and the timestamp of that measurement is shown with it.
- No invented availability, load, or server count. Catalogue values come from the CLI or are absent.

## Country metadata provenance

The fallback country table in `src/countries.rs` was synchronized from the installed `cyberghostvpn-gui` 1.0.7 settings cache. It contains 100 unique country codes and names.

Additions made during synchronization included Bolivia, Dominican Republic, Ecuador, Guatemala, Laos, Myanmar, Nepal, Peru, and Uruguay, plus aliases for names such as Bosnia and Herzegovina, Burma, Macau, and South Korea. Alias matching runs before general country-name matching so official compound names such as "Macau SAR China" resolve to Macau rather than China.

The fallback is not a server list, and it is not a claim that every country has every server type. Torrent country availability is read separately from the CLI and can be smaller than the traffic list.

## Verification status

Completed locally:

- `cargo fmt`, `cargo test`, debug build, and release build.
- Unit tests for CLI table parsing, status parsing, connect argument construction, profile IDs, city instance counts, fallback country uniqueness, the profile parser, NetworkManager parsing, and local storage.
- Live Traffic country table: 100 countries.
- Live Torrent country table: 86 countries.
- Live US streaming service table: 29 services.
- Live country, city, and server navigation for Traffic, Torrent, and Streaming.
- Streaming flow with service names containing spaces, including `Amazon Prime US`, `Apple TV US`, and `Fandango US`.
- Saving a real CyberGhost selection from Network to an isolated Locations state, with `state.json` at `0600` and its directory at `0700`, then reloading it after a normal restart.
- External tunnel detection, honest transport and endpoint placeholders, and a normal app exit that left the external tunnel running. Save and Connect stay disabled when no location is selected, and Connect stays disabled while an external tunnel is active.
- Labelled fallback behavior for a simulated 401 catalogue failure and for a missing CLI. In both cases city and server lists stayed empty and actions were disabled.
- UI verification in light and dark themes, including Network, Locations, Activity, Settings, no-profile Connect, and the external tunnel session view.

Not completed:

- Opening a new CyberGhost tunnel from Ghostline with a real account. An external tunnel was active and was not disturbed.
- Importing and activating a real profile through NetworkManager on this machine.
- Testing on non-Linux platforms.
- A full review of every telemetry event in every official build.

The demo mode in the application is deliberately isolated. It does not prove that a production tunnel works.

## Artifact handling

The proprietary installer and extracted payload are research inputs. They are not part of Ghostline's source distribution and must not be published with it. The installer is excluded in `.gitignore`. Ghostline's source and its bundled fonts and country metadata have their own licenses, recorded in `LICENSE` and `THIRD_PARTY_NOTICES.md`.
