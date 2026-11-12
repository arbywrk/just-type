# Just Type

This is a little typing game for practicing touch typing. I made it for fun and it is a bit rough around the
edges, but if you like it feel free to play and contribute.

# Building

> Note: building it from source necessitates some dependencies which are not documented outside the nix flake.

```bash
# for debug builds just omit the release flag
cargo build --release
```

# License

Just Type is free software: you can redistribute it and/or modify it under the terms of the GNU General
Public License as published by the Free Software Foundation, either version 3 of the License, or (at your
option) any later version. See [LICENSE](LICENSE) for the full text.

# Credits

Inspired by [Monkeytype](https://github.com/monkeytypegame/monkeytype), which shaped how the app behaves
and its minimal style. The English word list (`resources/languages/english.json`) comes from Monkeytype,
licensed under [GPL-3.0](https://github.com/monkeytypegame/monkeytype/blob/master/LICENSE).

Icons are from [Phosphor Icons](https://github.com/phosphor-icons/core), licensed under
[MIT](https://github.com/phosphor-icons/core/blob/main/LICENSE).

The font is [JetBrains Mono](https://github.com/JetBrains/JetBrainsMono), licensed under
[OFL-1.1](https://github.com/JetBrains/JetBrainsMono/blob/master/OFL.txt).
