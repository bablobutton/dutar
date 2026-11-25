## Bugs

## Rough plan of things to do in no particular order

* Support creating playlists.
* Some way of displaying and playing albums, playlists, artists. - depends on playlists.
* Shuffle, repeat - depends on playlists.
* Album cover -> ASCII art
* "Schedule next" hotkey. - depends on playlists.
* Make OS recognize dutar as music player so that headset integration works, multi media keys, so on. - this is so non trivial and technically limited if done multiplatform. E.g. macos complicates this if done in a terminal app and not a gui.
* Volume is a bit too loud on 5%. Need different volume scaling.

* Search
    - Need fuzzy search among all fields at once - artist, title, album.
    - Search bar should have a dropdown with fuzzily matched songs. Then select with C-n/C-p.
    - I think search should only work on currently opened playlist.
    This way we can remove "ClearSearch" command and not change the list of songs in the UI.
    But for that we need to implement playlists.
    - Can hit "Schedule next" from search bar for a matched song without closing the search bar.

* Layout-agnostic keypresses. Bind actions to keys, not characters. So hotkeys still work in Russian layout.
    - this needs to be supported at crossterm level but they haven't implemented kitty protocol for this yet
    - https://docs.rs/crossterm/latest/crossterm/event/struct.KeyboardEnhancementFlags.html
    - we can't just create mapping йцукен -> qwerty due to some edge cases (e.g. dvorak)
    - workaround for this could be implementing platform support for media keys - but this is hell on its own.
    - best is to fork crossterm and implement what we need.

* Removing/adding to queue. Should work via search mechanism - search should be done across all available songs, sort of a global playlist.
* The Artist, Title, Album names of the song currently in focus move so you can see all of the data
* Some way to scroll list without switching songs (<C-d>/<C-u> like vim? ).
    - Or, alternatively we could make <C-n>/<C-p> to change selection instead of switching songs. Selected song can then be played when hit say <Enter>. <C-d>/<C-u> like vim can still be added.

## Concurrency:
* Pure ELM clearly stopped working for us because we have to react to timer events now. See commit message in 66aaa935db.
* It's worth looking into how tokio fits, but we can also just have 2 static threads with a shared event queue.
One thread would read state -> render UI -> dispatch events into a queue and the other would process the event queue. I'm thinking this is simpler than bringing whole async/await concept.
* However, how well will this fit into detached UI strategy that we'll definitely need in the future? I want dutar to be a background service and with an attachable UI client that just renders the state. But even here I think static threads and shared queues are fine.

...

* P2P network - ? Do we even need this? It's intended to allow searches and downloads between the users, so a centralized collection is not needed. But maybe an over complication - we can just let users handle their own download method.

* Sync with user-provided cloud drive - ? gdrive, yandex, s3? This would solve most use cases of p2p network. I think we should have this before multi-platform support.

* You know how steam can offer to dowload missing data from devices on the same LAN? We can do that too - automatically expose access to libraries of users on the same network and offer user to download missing songs from them.
    - Later, the same sharing can work over internet - given that users have "friended" each other. Though that will require some sort of central server (either server-client or p2p NAT).

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
