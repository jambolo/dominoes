# dominoes

Play Dominoes against the computer.

## Implementation

### Rollout Heuristics

| Rank                                                      | Heuristic                                                                                                                                                                                    |
| --------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1. Tile Tracking & End-Frequency Awareness                | The single strongest predictor of expert or AI strength. Knowing which numbers are “live” or “dead” underpins nearly every other decision. Without this, all other heuristics are guesswork. |
| 2. Mobility / End-Control (Maintain Initiative)           | Having the ability to play to multiple ends is the key to avoiding forced passes. Agents maximizing “valid move count” consistently outperform others.                                       |
| 3. Minimize Pip Count (Safe Reduction)                    | Especially decisive in draw/block variants — losing by pip total is common. Optimal players shed large tiles unless it harms control.                                                        |
| 4. Opponent End Restriction (Forcing Passes)              | Steering the board toward numbers the opponent lacks yields direct tempo control and block wins. Second only to tile tracking in late-game value.                                            |
| 5. Balanced End Composition (Avoid Single End Dependence) | Reduces chance of total block; a major mid-game stability factor.                                                                                                                            |
| 6. Early High-Tile Play (When Safe)                       | A consistent gain in expected pip differential; reduces high-pip traps.                                                                                                                      |
| 7. Endgame Lookahead / Minimax Pip Outcome                | Crucial in final 4–6 tiles; top bots explicitly simulate this.                                                                                                                               |
| 8. Double-Tile Timing (Hold Common, Shed Rare)            | Expert hallmark. Doubles control tempo, but dead doubles sink win rate if held too long.                                                                                                     |
| 9. Board Closure & Block Construction                     | Knowing when to close an end versus expand one improves control in low-mobility phases.                                                                                                      |
| 10. Forcing Single-End Scenarios (When Ahead)             | Converts initiative into deterministic wins. Risky if misapplied.                                                                                                                            |
| 11. Tempo Sacrifice for Strategic Control                 | Used by advanced players to manipulate future ends. Requires foresight.                                                                                                                      |
| 12. Probe Plays for Information                           | Marginally improves inference; valuable in hidden-hand variants.                                                                                                                             |
| 13. Countdown Simulation (Explicit Small-Tree Search)     | When near the end, limited lookahead produces measurable gains but is computationally heavy.                                                                                                 |
| 14. Create Forks (Branch Opportunities)                   | Helpful early, but loses value if opponent tracks tiles well.                                                                                                                                |
| 15. Pip Sum Steering (High vs Low Ends)                   | Secondary pip optimization; relevant in scoring variants more than in block.                                                                                                                 |
| 16. Ambiguity Maintenance                                 | Bluffing and concealment have small effect under perfect tracking.                                                                                                                           |
| 17. Early Game Diversity (End Variety Maximization)       | Useful but dominated by mobility and pip minimization metrics.                                                                                                                               |
| 18. Fork Avoidance When Losing Control                    | Important only when under pressure; otherwise redundant with mobility control.                                                                                                               |
| 19. Tempo Switching via Doubles                           | A stylish but minor tactic unless the board is symmetric.                                                                                                                                    |

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
