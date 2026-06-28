# YAN Support for JetBrains IDEs

JetBrains IDEs (IntelliJ IDEA, WebStorm, PhpStorm, PyCharm, GoLand, CLion,
RubyMine, Rider, etc.) can directly import a **VS Code extension folder**
as a TextMate bundle for syntax highlighting — no separate plugin needed.

This reuses `editors/vscode/` (the same folder built for VS Code), since
it already has a `package.json` and a TextMate grammar.

## Install

1. Open **Settings/Preferences** (`Ctrl+Alt+S`)
2. Go to **Editor → TextMate Bundles**
3. Make sure the **TextMate Bundles** plugin is enabled (bundled by
   default in most JetBrains IDEs; install from Marketplace if missing)
4. Click **+** and select the `editors/vscode` folder from this repo
5. `.yan` files will now get syntax highlighting

## Limitations vs. a native plugin

This gives syntax highlighting only (same TextMate grammar as VS Code).
It does **not** provide:
- Code completion / IntelliSense
- Inline error checking (PSI-based inspections)
- Refactoring support
- Custom file icon in the project tree

A full native plugin (written in Kotlin against the IntelliJ Platform
SDK) would be needed for those features. That is a significantly larger
project — open an issue if there's real demand for it before investing
in a full PSI-based language plugin.
