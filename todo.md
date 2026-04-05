## Rough plan of things to do in no particular order

* Support creating playlists.
* Command bar.
    - dropdown with suggestions filtered by fuzzy search
    - select from dropdown on C-n and C-p
    - autocomplete from dropdown selector on TAB press or enter
* Some way of displaying and playing albums, playlists, artists.
* Shuffle, repeat
* Album cover -> ASCII art
* Exponential (or quadradtic) skipping intervals when holding skip button (J or L). Example: holding L should skip forward 1s, 2s, 4s, 8s, 16s...
* Set duration on number keys like youtube: 0 -> start of song, 1 -> 10%, 2 -> 20%, ...
* Double Ctrl+C to quit the app instead of 'q' with a warning after the first hit.
* "Schedule next" hotkey.
* Make OS recognize dutar as music player so that headset integration works, multi media keys, so on.

* Search
    - Need fuzzy search among all fields at once - artist, title, album. Can probably reuse ripgrep.
    - Search bar should have a dropdown with fuzzily matched songs. Then select with C-n/C-p.
    - I think search should only work on currently opened playlist.
    This way we can remove "ClearSearch" command and not change the list of songs in the UI.
    But for that we need to implement playlists.
    - Can hit "Schedule next" from search bar for a matched song without closing the search bar.

* Layout-agnostic keypresses. Bind actions to keys, not characters. So hotkeys still work in Russian layout.
    - this needs to be supported at crossterm level but they haven't implemented kitty protocol for this yet
    - https://docs.rs/crossterm/latest/crossterm/event/struct.KeyboardEnhancementFlags.html
    - we can't just create mapping йцукен -> qwerty due to some edge cases

...
P2P network - ?


## Side quests

The major use case for CLI apps is not actually using it as a full-blown app but as a tool.
Hence, we may want to think about additional features user can do with `dutar` executable with CLI interface (not TUI).
Here are some draft ideas:

* Produce *cool* ascii art from images or songs containing album covers.
* Download from online (ex. youtube).
* Encoding conversion.
* Read/Write metadata (format recongnition, duration, titles).
* Play a song but not occupy entire terminal screen, just a few rows below the shell cmd line `$` with name, duration, and a little oscilloscope.
* Trim duration.
