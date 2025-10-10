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

### Generate

The `generate` utility randomly generates and prints a dominoes layout for a given set and variation.

#### Command Line Syntax

```bash
generate [OPTIONS] [SIZE]
```

##### Arguments

- `[SIZE]`: Maximum size of the layout (number of tiles). Optional; defaults to the full set size.

##### Options

- `-s, --set <SET>`: Domino set to use (e.g., 6 for double-six, 9 for double-nine). Optional.
- `-v, --variation <VARIATION>`: Game variation to use (e.g., traditional, allfives, allsevens, bergen, blind, fiveup). Optional.
- `-j, --json`: Output in JSON format (not yet implemented).
- `-h, --help`: Print help information.
- `-V, --version`: Print version information.

#### Example Usage

Generate a random layout using the default set and variation:

```bash
generate
```

Generate a random layout of up to 10 tiles from a double-nine set:

```bash
generate 10 --set 9
```

Generate a random layout using the "allfives" variation:

```bash
generate --variation allfives
```
