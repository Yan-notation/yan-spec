# YAN Language Support for VS Code

Syntax highlighting and a file icon for `.yan` (YAN — Yet Another Notation) files.

## Features

- Syntax highlighting for comments (`#`, `/* */`), strings, numbers,
  booleans (`true/false/yes/no/on/off`), null (`null/nil/Nil/_`),
  type hints (`@int`, `@date`, `@url`, etc.), and keys
- Comment toggling (`Ctrl+/`) and bracket matching for `{ }`
- A custom file icon for `.yan` files in the Explorer

## Installing the file icon

⚠️ **Known limitation**: VS Code icon themes are all-or-nothing — once you
activate this extension's icon theme, files *other* than `.yan` will show
no icon in the Explorer (this is a VS Code platform limitation, not a bug
in this extension). If you want full coverage for all your other file
types too, use this alongside a general-purpose icon pack instead of as
your only icon theme, or only enable it when working specifically on YAN
files.

To enable the icon:
1. Open Command Palette (`Ctrl+Shift+P`)
2. Run **Preferences: File Icon Theme**
3. Select **YAN File Icon**

## Local development / testing

```bash
cd editors/vscode
npm install -g @vscode/vsce   # if not already installed
vsce package
```

This produces a `.vsix` file you can install locally via:
```bash
code --install-extension yan-notation-1.0.0.vsix
```

## Publishing

Requires a [Visual Studio Marketplace publisher account](https://marketplace.visualstudio.com/manage):
```bash
vsce login yan-notation
vsce publish
```
