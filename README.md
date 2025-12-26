# dutar - Terminal Music Player
```
                    @                   
                    @                   
                    @                   
                                        
                    @                   
                    @                   
                    @                   
                    @                   
                    @                   
                    @                   
                    @                   
                    @                   
                    @                   
                    @                   
                    @                   
                    =                   
                    :                   
                                        
                    @@                  
                   @@@                  
                   @.@                  
                  @@:#@                 
                 @#%.-@@                
                 @+@  @+@               
                @@:@@#@ @               
                @@:#@-@:@               
                 @@@@@@@                
                    @                   
```

## Features
- TUI interface
- Keyboard driven workflow

## Installation
```shell
cargo install --locked --path .
# or in debug mode with logs
cargo install --debug --locked --path .
```
Installs into `~/.cargo/bin/`

## Usage
- `dutar` - will run with queue from previous session loaded. If no previous queue,
will load all songs from system default Music directory (ex. `~/Music`).
- `dutar ~/some/dir` - will run with songs from user-specified directory.
- `dutar ~/some/dir/song.mp3` - will with a single song open.

## Environment variables
- `LOG_PATH` - specify a file path to write logs to. When not provided, logs are written system's local data directory:
```
Linux:   ~/.local/share/dutar/dutar.log
macOS:   ~/Library/Application Support/dutar/dutar.log
Windows: C:\Users\<user>\AppData\Roaming\dutar\dutar.log
```
Example: `LOG_PATH=/tmp/my_logs.log dutar`
Logs are only written in debug build.

## Fun facts 
Named after the **dutar** - a traditional Turkmen two-stringed instrument.
