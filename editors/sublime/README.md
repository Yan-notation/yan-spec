# YAN Syntax for Sublime Text

Syntax highlighting for `.yan` (YAN — Yet Another Notation) files.

## Install (manual)

1. Open Sublime Text → **Preferences → Browse Packages...**
2. Create a folder named `YAN` inside the Packages directory
3. Copy `YAN.sublime-syntax` and `Comments.tmPreferences` into it
4. Restart Sublime Text, or run **View → Syntax → YAN**

## Install (Package Control)

Not yet published to Package Control. To publish:
1. Push this folder to its own repo (or keep it as a subfolder reference)
2. Submit to [Package Control](https://packagecontrol.io/docs/submitting_a_package)

## Features

- Comments (`#` line, `/* */` block) with `Ctrl+/` toggle support
- Strings, numbers, booleans (`true/false/yes/no/on/off`)
- Null aliases (`null`, `nil`, `Nil`, `_`)
- Type hints (`@int`, `@date`, `@url`, etc.)
- Key highlighting
