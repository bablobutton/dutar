## Bugs

* if holding skip button (L or J) it would keep skipping even after you release. It's probably processing too many presses. Can even result in skipping multiple songs. Same when holding (N or J) - seems like it'll keep skipping even after you release.
We should probably have some general solution for holding the keys. Maybe ignore extra presses in a given time window?
Interesting to look how neovim handles that because they probably do.



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
    - workaround for this could be implementing platform support for media keys - from above.

* Ability to repeat songs
* Hide songs from this session without actualy deleting them
* The Artist, Title, Album names of the song currently in focus move so you can see all of the data
* Some way to scroll list without switching songs (<C-d>/<C-u> like vim? ).
    - Or, alternatively we could make <C-n>/<C-p> to change selection instead of switching songs. Selected song can then be played when hit say <Enter>. <C-d>/<C-u> like vim can still be added.

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
