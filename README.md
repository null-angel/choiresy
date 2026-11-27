# Choiresy

A small Rust desktop and system-tray app for a community-owned AI music library.
Add creator, title, and IPFS CID metadata to keep a local catalog of tracks; the
library is saved to `~/.choiresy/library.json` (or
`%USERPROFILE%\.choiresy\library.json` on Windows). Closing the window hides
the app in the system tray.

## Run

```sh
cargo run
```

The app is an early desktop foundation: it stores track metadata locally and
does not yet connect to IPFS, play audio, or implement on-chain features.
