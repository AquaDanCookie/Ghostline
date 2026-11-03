# Third-party notices

Ghostline's own source is licensed under the MIT License in `LICENSE`.

## Country metadata

The country code and name table in `src/countries.rs` was synchronized from the settings cache of `cyberghostvpn-gui` (https://github.com/picharly/cyberghostvpn-gui), an unofficial MIT-licensed client. No profiles, images, translations, or other assets from that project are included.

Live country, city, server, streaming service, and load tables are read from the official CyberGhost CLI at runtime. They are not bundled in this repository.

Upstream copyright and license:

```
Copyright (c) 2024 Charly Pichard

Permission is hereby granted, free of charge, to any person obtaining a copy of
this software and associated documentation files (the "Software"), to deal in
the Software without restriction, including without limitation the rights to
use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies
of the Software, and to permit persons to whom the Software is furnished to do
so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

## Fonts

The bundled IBM Plex Sans and IBM Plex Mono files are Copyright 2017 IBM Corp. with Reserved Font Name "Plex". They are licensed under the SIL Open Font License, Version 1.1. The full license is in `assets/fonts/OFL.txt`.

## Rust dependencies

Rust dependencies are listed in `Cargo.lock` and retain their own licenses. They are not vendored in this repository. Run `cargo metadata` or inspect the crate sources for the exact license of each dependency.

## CyberGhost artifacts

The proprietary CyberGhost Windows installer and any extracted payload are research inputs and are not distributed with Ghostline. CyberGhost and its logos are trademarks of their respective owner. Ghostline is an independent project and is not affiliated with CyberGhost.
