# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**dutar** is a TUI (Terminal User Interface) music player written in Rust. Named after the traditional Turkmen two-stringed instrument, it provides a keyboard-driven workflow for playing local music libraries.

## Development Commands

### Building and Running
- `cargo build` - Compile the project
- `cargo run` - Build and run the application
- `cargo build --release` - Build optimized release version
- `cargo check` - Quick compile check without producing binary

### Testing
- `cargo test` - Run all tests
- `cargo fmt --check` - Check code formatting (pre-commit hook)

### Code Quality
- `cargo fmt` - Format code
- `cargo clippy` - Run linter

## Architecture

### ELM Architecture Pattern
The application follows the [ELM architecture](https://ratatui.rs/concepts/application-patterns/the-elm-architecture/) pattern with three core components:

1. **Model** (src/main.rs:18-51) - Application state container including:
   - `app_state`: Current playback state (Init, Player, Done)
   - `popup`: Optional popup state (command bar, hints)
   - `audio`: Rodio audio output stream and sink
   - `queue`: Song queue management
   - `channel`: Message passing for async events

2. **View** (src/tui.rs, src/tui/) - Rendering logic using Ratatui:
   - Main UI limited to 100x30 centered area
   - Queue table with scrollbar
   - Volume and progress gauges
   - Popup overlays (command bar in src/tui/bar.rs, hints in src/tui/hints.rs)

3. **Update** (src/main.rs:214-348) - State transitions via message handling:
   - Message enum defines all possible actions
   - `update()` function processes messages and returns optional follow-up messages
   - Event loop in `main()` handles keyboard input and channel messages

### Key Modules

- **src/main.rs** - Main event loop, state management, message handling
- **src/controls.rs** - Audio playback controls (play, pause, skip, volume)
  - Uses Rodio for audio decoding and playback
  - Implements callback-based auto-play-next via empty source trick
- **src/queue.rs** - Music library scanning and queue management
  - Scans `~/Music` (or system audio directory) on startup
  - Extracts metadata using `lofty` crate
  - Supports advance/retreat with looping
- **src/tui.rs** - Main rendering logic and layout
- **src/tui/bar.rs** - Command bar popup rendering
- **src/tui/hints.rs** - Keyboard hints popup rendering
- **src/utils.rs** - Utility functions for metadata extraction and directory scanning
- **src/logging.rs** - Logging initialization using env_logger

### State Flow

```
Init -> Player(Playing/Paused) -> Done
         ^                   |
         |    Messages       |
         +-------------------+
```

Messages flow through:
1. Keyboard events -> `handle_key()` -> Message
2. Channel events (e.g., song ended) -> Message
3. `update()` processes Message -> state transition -> optional new Message

### Audio System

- Uses `rodio` crate for audio output
- `OutputStream` must be kept alive (stored in Model)
- `Sink` handles playback queue and controls
- Auto-play-next implemented via `EmptyCallback` source (src/controls.rs:50-72)
- 5MB buffer for file reading

### Command System

Command bar (triggered by `:`) supports:
- `quit`, `q` - Exit application
- `play`, `pause`, `toggleplay` - Playback controls
- `next`, `prev` - Track navigation
- `setv <0-100>` - Set volume
- `mute`, `unmute`, `togglemute` - Mute controls

### Keyboard Shortcuts

- `k` - Toggle play/pause
- `l` / `j` - Skip forward/backward 5 seconds
- `n` / `p` - Next/previous track
- `+`, `=` / `-`, `_` - Volume up/down
- `m` - Toggle mute
- `:` - Open command bar
- `?` - Show hints
- `q` - Quit (when no popup open)

## Testing Notes

- Tests are in src/main.rs (line 411+)
- Current tests are commented out due to audio initialization requiring actual music files
- Tests focus on state transitions in the `update()` function
- Consider mocking audio system for testability

## Git Hooks

- **pre-commit**: Runs `cargo fmt --check`
- **pre-push**: Runs `cargo test`
- Configured via `.rusty-hook.toml`

## Dependencies

Key crates:
- `ratatui` - TUI framework
- `crossterm` - Terminal manipulation
- `rodio` - Audio playback
- `lofty` - Audio metadata extraction
- `symphonia` - Audio codec support
- `color-eyre` - Error handling
- `infer` - File type detection
