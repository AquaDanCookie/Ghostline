# Privacy and data map

Ghostline is a local desktop client. It has no account system, telemetry, analytics, crash reporting, advertising, upsells, or remote configuration. It does not check for updates.

Ghostline does use the official CyberGhost CLI. That matters for the data map: catalogue, connection, and status commands leave Ghostline and run in the CLI, which has its own account session and network behavior. This document separates what Ghostline does directly from what the official CLI does when Ghostline asks it to act.

This document describes the current version. It is a map, not a promise about future versions.

## What leaves this machine

Ghostline makes no automatic analytics, update, or feature-flag requests. The actions below can use the network. Each one is either a VPN connection you configured, an official CLI command, or a button you pressed.

| Action | Destination | What is sent | When |
| --- | --- | --- | --- |
| Open Network, select a server type, country, or streaming service | CyberGhost, through the official `cyberghostvpn` CLI | The catalogue query and the account session already stored by the CLI | When you open Network or change the selection that requires a new list |
| Check tunnel status | The local `cyberghostvpn --status` command | No catalogue request. The CLI checks the local connection state | At startup and while a tunnel is active |
| Connect or stop a CyberGhost location | CyberGhost and the selected VPN endpoint, through the official CLI with `sudo` | The CLI command, account session, chosen country, city, server, streaming service, protocol, and VPN traffic | When you press Connect or Disconnect, or when auto-connect or automatic reconnect is enabled |
| Import a profile | Local NetworkManager | The selected `.ovpn` or `.conf` file | When you confirm an import |
| Connect or disconnect an imported profile | Local NetworkManager, then the VPN endpoint in the profile | VPN protocol traffic and credentials as required by the profile | When you press Connect or Disconnect, or when auto-connect or automatic reconnect is enabled |
| Exit IP check | `https://api.ipify.org?format=json` over HTTPS | A request with the user agent `Ghostline/0.1 exit-ip-check` | Only when you press Check exit IP |
| Measure latency | The host and port in the selected imported profile | A TCP handshake with no application payload, up to three attempts | Only when you press Measure latency |
| Refresh network details | Local `nmcli`, `/etc/resolv.conf`, `resolvectl`, and `ip` | Nothing remote | When the app starts or you refresh |
| Flush DNS | Local system resolver | Nothing remote | Only when you press Flush DNS |

The official CLI may contact CyberGhost for account and catalogue work even when Ghostline only asks for a list. Ghostline does not read, copy, or store the CLI's account token. The CLI's own data handling is outside Ghostline and is governed by the official client and CyberGhost's terms.

The exit IP response is held in memory and shown in the interface. It is not written to the state file or the activity log. A latency result is stored locally with the imported profile because it is a measurement you asked Ghostline to keep.

The VPN traffic itself is carried by CyberGhost's OpenVPN or WireGuard service or by NetworkManager and the selected plugin. Ghostline does not proxy, inspect, or record tunnel traffic. It reads interface counters from `/sys/class/net` to display transfer totals.

## Files Ghostline reads and writes

| Path | Purpose | Permissions |
| --- | --- | --- |
| `$XDG_CONFIG_HOME/ghostline/state.json` | Saved CyberGhost and imported locations, NetworkManager UUIDs, source paths, usernames, favorites, measured latency, window size, settings | `0600` |
| `$XDG_DATA_HOME/ghostline/logs/ghostline.log` | Local activity log | `0600` |
| `$XDG_DATA_HOME/ghostline/logs/ghostline.log.1` | Previous log after rotation | `0600` |
| `$XDG_CONFIG_HOME/autostart/ghostline.desktop` | Optional desktop autostart entry | `0644` |
| `$XDG_RUNTIME_DIR/ghostline/passwd-*.tmp` | One-time NetworkManager credentials file for imported profiles | `0600` |
| `$XDG_CONFIG_HOME/ghostline/state.json.bad` | Copy of an unreadable state file, when recovery fails | `0600` |

On Linux, `$XDG_CONFIG_HOME` normally defaults to `~/.config`, `$XDG_DATA_HOME` to `~/.local/share`, and `$XDG_RUNTIME_DIR` to a private per-user runtime directory. If `XDG_RUNTIME_DIR` is unset, the credentials file is created under the system temporary directory in a `ghostline` subdirectory with owner-only permissions.

Ghostline creates its own configuration, data, log, and runtime directories with `0700`. It does not change the permissions of the shared desktop autostart directory.

The state file does not contain account passwords, CyberGhost tokens, or the sudo password. It may contain usernames, profile paths, endpoint host names, NetworkManager UUIDs, CyberGhost country and city names, streaming service names, server instance names, load values reported by the CLI, and settings. An unreadable state file is copied to `state.json.bad` before defaults are used, so that copy has the same metadata.

The activity log records connection events, profile changes, sudo prompts, and errors. It does not record passwords or tunnel payloads. The log rotates at roughly 1 MiB. The default retention is 30 days, checked at startup. Clearing the activity log removes both the current file and the rotated file.

Catalogue list responses are held in memory while Ghostline runs. Ghostline does not write the full country, city, server, or streaming service tables to disk. Only your saved selection is stored in `state.json`.

## Credentials and sudo

Ghostline never asks for a CyberGhost account or password. Sign-in happens through the official CLI. The CLI stores and owns its account session; Ghostline does not read that file or reuse the token directly.

CyberGhost connect and stop operations may need administrator rights. Ghostline runs them through `sudo`.

- The sudo password is written to `sudo` over stdin with `sudo -S --`.
- It is never put in process arguments, Ghostline's state file, or the activity log.
- By default it is kept only long enough to complete the command.
- If you enable **Keep the administrator password**, it stays in memory for this process so later CyberGhost actions do not ask again. It disappears when Ghostline exits.
- The official CLI may also write privileged files or logs as part of its own operation. Those files belong to the CLI and the system, not to Ghostline.

Imported profile credentials are separate from sudo:

- If you choose **Save password in the system keyring**, the password is stored through `secret-tool` under the service `ghostline` and the profile ID. The username is stored in the state file so it can be filled in next time.
- If you do not choose that, the password is not written to Ghostline's state file or log. NetworkManager receives it through a temporary credentials file with `0600` permissions. Ghostline deletes that file after the `nmcli` call returns.
- The backend keeps imported credentials in memory for the current run so automatic reconnect can use them. They are not written to disk and disappear when the process exits.
- Removing an imported profile clears its keyring entry.
- If Ghostline is killed while `nmcli` is running, the temporary credentials file may remain in the runtime directory. It is still owner-only and can be removed manually. The runtime directory is normally cleared at logout.

The keyring is provided by the desktop secret service. Ghostline does not implement its own encryption or password storage.

## Catalogue fallback boundary

Ghostline shows live catalogue data only when the official CLI returns it. If the CLI is missing or a list request fails before any successful response, Ghostline may show the reference country table in `src/countries.rs`.

That fallback is limited by design:

- It is labelled as reference data in the interface.
- It is used only for country navigation.
- It never gains invented cities, servers, streaming services, load values, or latency.
- Save and Connect actions are disabled while the fallback is in use.
- If a previous live list is still available, Ghostline says the displayed list is the last successful response instead of replacing it with reference data.

## Clipboard

Copy actions place an endpoint or log text on the desktop clipboard. Clipboard history, synchronization, and persistence are controlled by your desktop environment, not by Ghostline.

## Demo mode

Demo mode is visibly labelled. It uses sample profiles and a sample catalogue under the system temporary directory with simulated tunnel, transfer, latency, and exit IP values. It does not call `nmcli` or `cyberghostvpn`, read or change real NetworkManager connections, or modify real Ghostline settings, logs, autostart entries, or network traffic.

## Third-party data

Ghostline includes IBM Plex fonts under the SIL Open Font License. The license text is in `assets/fonts/OFL.txt`.

The fallback country code and name table was synced from the MIT-licensed `cyberghostvpn-gui` project. Live country, city, server, and streaming service tables are read from the official CLI at runtime and are not bundled. Ghostline does not bundle CyberGhost profiles, logos, icons, or other proprietary assets. See `THIRD_PARTY_NOTICES.md` for attribution.

## Removing local data

To remove everything Ghostline stores:

1. Remove saved CyberGhost locations and imported profiles in the app. Removing an imported profile also asks NetworkManager to delete its connection.
2. Clear the activity log in Settings.
3. Turn off Launch at login.
4. Delete `$XDG_CONFIG_HOME/ghostline` and `$XDG_DATA_HOME/ghostline`.
5. Remove any keyring entries for service `ghostline` if you saved imported profile passwords and did not remove the profiles in the app.

The original profile files you imported are yours and are not deleted by Ghostline. Your CyberGhost account session belongs to the official CLI. Use the CLI's own commands to sign out or remove its configuration. Removing Ghostline does not cancel a CyberGhost subscription or close a tunnel that the CLI opened.
