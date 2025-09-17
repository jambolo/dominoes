# dominoes

Play Dominoes against the computer.

## Utilities

Executables that demonstrate concepts and features.

### Visualize

The `visualize` utility provides functionality to visualize a text-format dominoes game layout in a graphical window or to export it as JSON.

#### Command Line Syntax

```bash
visualize [OPTIONS] <LAYOUT>
```

##### Arguments

- `<LAYOUT>`: The layout string to visualize (see examples below)

##### Options

- `-j, --json`: Print the layout as JSON to stdout instead of displaying graphically
- `-h, --help`: Print help information
- `-V, --version`: Print version information

#### Example Usage

Display a simple domino layout graphically:

```bash
visualize "3|3=(3|4-4|5,3|6)"
```

Export the same layout as JSON:

```bash
visualize --json "3|3=(3|4-4|5,3|6)"
```
