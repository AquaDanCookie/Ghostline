# Ghostline design direction

Working name: **Ghostline**
Working title only. The user can rename the product without changing the design system.

Reading this as: a native Linux network utility for privacy-conscious people who already use CyberGhost, in an instrument-panel visual language, dial ENERGY 2 / RHYTHM 2 / MOTION 1.

## Identity

Ghostline is the quiet layer between a person and their tunnel. It should feel engineered, not promotional: closer to a well-made network tool than a consumer security dashboard. The interface never sells anything. It reports what is true, gives one clear action per screen, and leaves the rest alone.

Personality: calm, precise, dry, slightly warm. Not corporate, not hacker-themed, not playful.

Anti-reference: the official client's stacked promotions, banner carousels, countdown offers, mascot illustrations, and telemetry consent screens. None of that belongs here.

## Dials

- **ENERGY 2**: present and confident, but not theatrical. One strong focal point per screen.
- **RHYTHM 2**: each screen has its own composition. Connect is asymmetric, Locations is a dense master-detail list, Settings is a single reading column.
- **MOTION 1**: motion only where it explains a state change. No ambient animation, no scroll reveals, no parallax.

## Palette

Neutral colors carry the interface. Amber is the single accent. Sage and rust are semantic only.

| Token | Dark | Light | Use |
| --- | --- | --- | --- |
| bg | `#0F1216` | `#F2EFE6` | window background |
| surface | `#171C21` | `#FBF9F3` | panels, list surface |
| surface_alt | `#222A31` | `#E8E3D7` | inputs, hover rows |
| line | `#2C343B` | `#D6CFC0` | separators, borders |
| text | `#F2EFE6` | `#171A1D` | primary text |
| text_muted | `#A6ADA9` | `#5C625F` | secondary text |
| accent | `#E9B44C` | `#A66A00` | one action, focus, active route |
| accent_ink | `#17130A` | `#FFF8E6` | text on accent |
| ok | `#82B78D` | `#256B3B` | connected state |
| danger | `#D9775C` | `#A33B21` | errors, destructive actions |

Why amber: it is the one visual thread back to CyberGhost's own yellow, used as a signal rather than a surface. It is not applied to every icon, border, and badge.

Why warm neutrals rather than blue-grey: the app is used in the evening and should not read as another cold developer console. The warmth also keeps the light theme from looking like a default settings dialog.

## Typography

- **IBM Plex Sans** for interface text. Humanist, engineered, highly legible at small sizes. Chosen over Inter and Geist because it has more character and a better technical temperament.
- **IBM Plex Mono** for IP addresses, ports, timings, protocol names, and log lines. Mono is used for data, never for headings or slogans.
- Scale: 12.5 caption, 14 secondary, 15 body, 17 emphasis, 22 section, 32 hero. Weights 400, 500, 600.
- Line height 1.45 for body, 1.2 for headings.

Both fonts are bundled under the SIL Open Font License in `assets/fonts/OFL.txt`.

## Layout

- 224 px navigation rail, 1 px separator, 40 px content padding. Content max width 1040 px.
- **Connect**: 7/5 split. Left column holds the connection control and route readout. Right column holds a compact session panel (duration, transferred, interface, protocol). The split is intentionally uneven.
- **Network**: three catalogue columns for country, city or streaming service, and server. Protocol controls sit above the catalogue. The selection summary sits below and shows exactly what the CLI will receive. Automatic city and server choices are explicit rows, not hidden defaults. Save and Connect stay disabled until a country is selected, and until a streaming service is chosen in Streaming mode.
- **Locations**: master-detail. Search and grouped profile list on the left, selected profile detail on the right. Dense rows, 56 px tall, not cards.
- **Activity**: a single log surface with a filter field and a copy action. No fake chart.
- **Settings**: one reading column, max 720 px, grouped rows with a label and a control. No card grid.
- **Import**: modal sheet with a drop target, file summary, and explicit credential choice.

The route line (two nodes joined by a line) is the identity motif. It appears in the connect hero, empty states, and the wordmark area. It is not repeated as decoration on every panel.

## Components

- Buttons: 10 px radius, 40 px minimum height, 44 px for primary actions. Primary is amber fill with ink text. Secondary is transparent with a 1 px line. Destructive is text-only rust until confirmation.
- Inputs: 10 px radius, surface_alt fill, 1 px line, 2 px amber focus ring.
- Navigation: text and a custom line icon, active state marked by amber text and a 2 px left rule. No pills.
- Toggle: 38 x 22 px track, 18 px thumb, amber when on, visible focus ring.
- Status: a 7 px dot plus a word. Color is never the only signal.
- Modal: 560 px max, surface fill, 1 px line, scrim at 55% black. Escape closes.
- Toasts: bottom right, surface fill, 1 px line, 4 second life, manual close.

## Iconography

Custom 1.5 px line icons drawn with egui primitives: power, route, search, activity, cog, import, copy, folder, trash, check, close, eye, eye-off, arrow-left. No icon library and no generic sparkle, star, shield-with-checkmark, or lightning bolt.

## Motion

- State changes: 160 to 220 ms ease-out.
- Connecting: a single dot travels along the route line, then stops. No looping after connection.
- Connected: a slow 2 second opacity pulse on the status dot only.
- Hover and focus: immediate color and stroke change, 120 ms.

## Copy rules

- Plain, specific, second person. "NetworkManager could not bring up the tunnel." not "Oops, something went wrong."
- No exclamation marks, no em dashes, no "seamless", "military-grade", "bank-grade", "AI-powered".
- Numbers only when measured or supplied by a verified source. Catalogue counts, server instances, and load values come from the official CLI. Never estimate latency, availability, load, uptime, or server counts.
- Empty states explain the next action. Errors name the failing component and the next step.
- Fallback data is labelled as fallback where it is visible, not only in documentation.

## Scope boundaries

- No account login. Account setup and sign-in stay in the official `cyberghostvpn` CLI. Ghostline runs the CLI for live catalogue, status, connect, and stop operations.
- No imported account tokens, scraped catalogue data, or bundled server lists. Live catalogue data is read from the CLI at runtime.
- No kill switch in version 1. A dead toggle would be worse than an honest omission, and a broken firewall rule can strand a user offline.
- No map. A map without real server coordinates is decoration.
- No telemetry, analytics, crash reporting, or remote feature flags, by construction.
- Ghostline makes no direct network request except the optional exit IP check and the optional latency test, both after a click. Catalogue, connection, and status work is performed by the official CLI, which has its own account session and network behavior.
- If the CLI is missing or a catalogue request fails, reference countries may aid navigation but never gain invented cities, servers, services, load values, or a connect action.

## Acceptance

- Every visible control has a working action.
- Empty, loading, error, connecting, connected, and disconnecting states exist for every data surface.
- Light and dark themes both pass WCAG AA for body text.
- All controls reachable by Tab, with a visible focus ring. Escape closes modals.
- The app builds and runs on Linux with NetworkManager and the OpenVPN and WireGuard plugins.
- The CyberGhost catalogue path is tested against the official CLI table formats, including repeated numeric row IDs in streaming server tables.
- A demo mode (`GHOSTLINE_DEMO=1`) exists for screenshots and click-through testing, is visibly labelled, and does not call `nmcli` or `cyberghostvpn`.
