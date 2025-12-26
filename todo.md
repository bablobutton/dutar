## Rough plan of things to do in no particular order

* Bug: play one song and delete another song from dutar dir. Need to detect when the song is missing and mark it as missing.
* Support creating playlists.
* Command bar.
    - dropdown with suggestions filtered by fuzzy search
    - select from dropdown on C-n and C-p
    - autocomplete from dropdown selector on TAB press or enter
* Search bar.
    - Same UI as for command bar with fzf and shit
* Some way of displaying and playing albums, playlists, artists.
* Shuffle, repeat
* Notification system (at least for errors that worth showing to user).
* Album cover -> ASCII art
* If current duration is > 3 seconds, "previus" action should restart the song
* Exponential skipping intervals when holding skip button (J or L). Example: holding L should skip forward 1s, 2s, 4s, 8s, 16s...
* Panic hook so that crashed app doesn't break the terminal.

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
