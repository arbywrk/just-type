# Just Type

This is a little typing game for practicing touch typing. I made it for fun and it is a bit rough around the
edges, but if you like it feel free to play and contribute.

# Building

> Note: building it from source necessitates some dependencies which are not documented outside the nix flake.

```bash
# for debug builds just omit the release flag
cargo build --release
```

# Where files are saved

Just Type keeps its settings in `settings.json` and your finished rounds in `history.sqlite3`.

| Platform      | Settings                                             | History                                  |
| ------------- | ---------------------------------------------------- | ---------------------------------------- |
| Linux         | `~/.config/just-type`                                | `~/.local/share/just-type`               |
| macOS         | `~/Library/Application Support/com.arbywrk.JustType` | same as settings                         |
| Windows       | `%APPDATA%\arbywrk\JustType\config`                  | `%APPDATA%\arbywrk\JustType\data`        |
| Debug builds  | `target/debug/data/config`                           | `target/debug/data/data`                 |

On Linux, `$XDG_CONFIG_HOME` and `$XDG_DATA_HOME` are respected. To use other folders, set
`JUST_TYPE_CONFIG_HOME` and `JUST_TYPE_DATA_HOME` to the folders the files should go in directly:

```bash
# to keep this persistent you can put them in your shell's rc file (.bashrc, .zshrc, etc.)
JUST_TYPE_CONFIG_HOME=/tmp/just-type JUST_TYPE_DATA_HOME=/tmp/just-type just-type
```

# License

Just Type is free software: you can redistribute it and/or modify it under the terms of the GNU General
Public License as published by the Free Software Foundation, either version 3 of the License, or (at your
option) any later version. See [LICENSE](LICENSE) for the full text.

# Credits

Inspired by [Monkeytype](https://github.com/monkeytypegame/monkeytype), which shaped how the app behaves
and its minimal style. The English word list (`resources/languages/english.json`) comes from Monkeytype,
licensed under [GPL-3.0](https://github.com/monkeytypegame/monkeytype/blob/master/LICENSE).

Icons are from [Cosmic Icons](https://github.com/pop-os/cosmic-icons) by [System76](https://system76.com/),
licensed under [CC-BY-SA-4.0](https://creativecommons.org/licenses/by-sa/4.0/).
