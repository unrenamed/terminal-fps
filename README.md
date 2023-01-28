[![](https://img.shields.io/github/license/unrenamed/terminal-fps)](https://img.shields.io/github/license/unrenamed/terminal-fps)

# terminal-fps

The port of [OneLoneCoder's](https://github.com/OneLoneCoder) [CommandLineFPS](https://github.com/OneLoneCoder/CommandLineFPS) written in C++ to Rust &#x1f980;

Everything was ported manually, no transpilers were used.

It works on both UNIX and Windows platforms thanks to cross-platform terminal manipulation library [crossterm](https://github.com/crossterm-rs/crossterm).

## Installation
Download the latest binary for your OS.

### Ubuntu
1. `cd` to the file you just downloaded
2. `dpkg -i <file-name>` will install it

## Usage
Before running the game, it's recommended to set your terminal size to 120 columns by 42 rows. You can try any font you like at size 11.

To run the game execute `terminal-fps` in your terminal. 
	
Controls: WASD or Arrow Keys

## Future modifications
1. Add an opportunity to pass a path to a .txt map via CLI, so that one can use it's own map built by either themselves or another program like [knossos](https://github.com/unrenamed/knossos)
