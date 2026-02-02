# Dominoes Game Rules

This document describes the rules for dominoes game variations supported by this codebase. Each variation shares common foundations but differs in scoring, spinners, and end-game conditions.

## Terminology

- **Tile**: A domino piece with two numbered ends (pips). Written as `a|b` where `a ≤ b`.
- **Double**: A tile where both ends have the same value (e.g., `3|3`).
- **Double-N Set**: A complete set where the highest pip value is N. Double-6 has 28 tiles; double-9 has 55 tiles.
- **Boneyard**: The pool of undrawn tiles.
- **Layout/Train**: The line of played tiles on the table.
- **Open End**: An end of the layout where tiles can be attached.
- **Spinner**: A double that allows tiles to be played on all four sides, creating branches.
- **Domino**: To play one's last tile (winning the hand).
- **Blocked Game**: No player can make a legal play.
- **Pass**: Skip a turn when unable to play (and drawing is not possible/required).

## Common Rules

### Tile Matching

Tiles are placed by matching pip values. A tile `a|b` can be placed on an open end showing `a` or `b`. The matching end connects; the other end becomes the new open end.

### Double Placement

Doubles are placed perpendicular to the layout (crosswise). Unless acting as a spinner, they still have only two open ends.

### Turn Order

Play proceeds clockwise. Players must play if able. Drawing and passing rules vary by variation.

---

## Variation: Traditional (Block/Draw)

The foundation game. Two sub-variants: **Block** (no drawing) and **Draw** (drawing allowed).

### Setup

| Players | Tiles per Hand | Boneyard |
|---------|----------------|----------|
|       2 |              7 |       14 |
|       3 |              7 |        7 |
|       4 |              6 |        4 |

### First Play

- **Option A**: Player with highest double leads with that double.
- **Option B**: Random lead; any tile may be played.

### Gameplay

- **Block Game**: If unable to play, pass. No drawing from boneyard.
- **Draw Game**: If unable to play, draw from boneyard until able to play or boneyard is reduced to 2 tiles (reserved). If boneyard exhausted, pass.

### Doubles

Doubles are placed crosswise but are **not spinners**. Only two ends remain open.

### End Conditions

1. A player dominoes (plays last tile).
2. Game blocks (no legal plays for any player).

### Scoring

- Winner: The domino player, or lowest pip-count holder in blocked games.
- Score: Sum of all opponents' remaining pips.
- Game target: Typically 100 points (2-player) or 61 points (3-4 player).

### Blocked Game Resolution

Compare total pips in each hand. Lowest total wins. Winner scores the difference between opponents' totals and their own.

---

## Variation: All Fives

Scoring game where points are earned during play for pip totals that are multiples of five.

### Setup

| Players | Tiles per Hand |
|---------|----------------|
|       2 |              9 |
|       3 |              7 |
|       4 |              5 |

### First Play

Highest double holder leads, or random determination.

### Gameplay

- If unable to play, draw until able or boneyard empty.
- **Cannot draw when able to play** (unlike Draw game).

### Spinner

The **first double played** becomes the spinner. Tiles can be placed on all four sides:

1. First two tiles must be placed on the two sides.
2. Third and fourth tiles extend from the ends.

Once both sides of the spinner have tiles, the spinner's pips no longer count toward the total.

### Scoring During Play

After each play, sum all open ends of the layout:

- If total is a multiple of 5 (5, 10, 15, 20, 25, 30, 35), score that many points.
- Doubles at an open end count their full pip value (e.g., `5|5` = 10).
- Maximum single-play score: 35 points.

### End of Hand Scoring

Sum opponents' remaining pips, round to nearest multiple of 5, and award to winner.

### Game Target

Typically 200-250 points (2 players) or 100-150 points (3-4 players). Cribbage board often used.

---

## Variation: All Sevens (Matador)

Also known as **Matador** or **Russian Dominoes**. Tiles match when their touching ends **sum to 7**, not by equality.

### Setup

| Players | Tiles per Hand |
|---------|----------------|
|     2-4 |              7 |

### Matching Rule

Instead of matching identical values, the pips on touching ends must sum to 7:

- 1 matches with 6
- 2 matches with 5
- 3 matches with 4
- 0 (blank) cannot be matched normally

### Matador Tiles

Four special tiles can be played on any open end:

- `0|0` (double blank)
- `1|6` (sums to 7)
- `2|5` (sums to 7)
- `3|4` (sums to 7)

After a matador is played, the next player must resume normal 7-sum matching or play another matador.

### Blank Handling

When a blank is at an open end, **only matadors can be played there**. This creates strategic blocking opportunities.

### Drawing

A player unable to play (and holding no matadors for blocked ends) must draw until able to play or boneyard is nearly exhausted.

### Optional Rule

A player holding a matador is not required to play it and may choose to draw instead.

### End Conditions

1. A player dominoes.
2. Game blocks entirely.
3. All players pass consecutively.

### Scoring

Lowest pip count wins. Winner scores sum of opponents' pips minus their own.

---

## Variation: Bergen

Scoring game based on matching the values at both ends of the layout.

### Setup

| Players | Tiles per Hand | Boneyard Reserved |
|---------|----------------|-------------------|
|       2 |              6 |                 2 |
|       3 |              6 |                 2 |
|       4 |              5 |                 2 |

### First Play

Player with **lowest double** leads with that double. If no doubles, reshuffle.

### Gameplay

- If unable to play, draw until able or boneyard reduced to 2 reserved tiles.
- Doubles are played crosswise but are **not spinners**.

### Scoring During Play

**Double Header (2 points):**
Both open ends of the layout show the same value.

```
Example: [3]===3|5===5|5 (both ends show 3 and 5|5 respectively - NOT a double header)
Example: [3]===3|5===5|3 (ends show 3 and 3 - DOUBLE HEADER)
```

**Triple Header (3 points):**

One end shows a double, and the other end matches its value.

```
Example: [3|3]===3|5===5|3 (double 3 on one end, single 3 on other - TRIPLE HEADER)
```

### Proximity Penalty

- Within 2 points of winning: headers score only 1 point each.
- Within 3 points of winning: headers score only 2 points each.

### End of Hand Scoring

The player who dominoes or wins a blocked game scores **2 points**.

### Blocked Game Resolution (American Rules)

Test in order until a winner is found:

1. Player with **no doubles** wins.
2. If all have doubles: player with **lowest double** wins.
3. If tied: player with **fewest tiles** wins.
4. If still tied: player with **lowest pip count** wins.

### Blocked Game Resolution (German Rules)

1. Player with no doubles wins.
2. Player with fewest doubles wins.
3. Player with lowest total pips wins.

### Game Target

- 2 players: 15 points
- 3-4 players: 10 points

---

## Variation: Blind (Blind Hughie)

A game of pure chance where players cannot see their own tiles.

### Setup

| Players | Tiles per Hand |
|---------|----------------|
|       2 |    8 (or 7-14) |
|       3 |              7 |
|       4 |              6 |

Tiles are drawn face-down and arranged in a row in front of each player. **Players do not look at their tiles.**

### First Play

First player turns over the rightmost tile in their row and plays it to start the layout.

### Gameplay

1. On your turn, flip the rightmost tile in your face-down row.
2. If playable: play it to the layout.
3. If not playable: flip it back face-down and move it to the left end of your row.
4. **Exception**: Doubles remain face-up when flipped. A player may play an exposed double instead of drawing from their row.

### No Drawing

There is no boneyard drawing. All tiles are distributed at setup.

### End Conditions

1. A player plays all tiles.
2. Game blocks (no player can make a legal play).

### Scoring

Lowest pip count wins. Winner scores sum of opponents' pips minus their own.

### Game Target

Typically 100 points (2-player) or 61 points (3-4 player).

### Memory Variant

Instead of flipping tiles in order, players may choose any face-down tile. Previously seen tiles provide memory advantage.

---

## Variation: Five Up

The most complex scoring variant. **All doubles are spinners**, creating tree-like layouts.

### Setup

| Players | Tiles per Hand |
|---------|----------------|
|       2 |            5-7 |
|       3 |              5 |
|       4 |              5 |

Best played as 4-player partnership game.

### First Play

First player may play any tile. If a double, it immediately becomes a spinner.

### Spinners (All Doubles)

Every double played is a spinner. Play against a spinner follows this order:

1. First tile: against one side.
2. Second tile: against the opposite side.
3. Third tile: extending from one end.
4. Fourth tile: extending from the other end.

Once tiles are placed on both sides of a spinner, **the spinner's pips no longer count** toward the total, though its ends remain open.

### Scoring During Play

After each play, sum all open ends of the layout:

- If total is a multiple of 5, score that value divided by 5.
- Example: Total of 15 = 3 points.

Because all doubles are spinners, layouts can have many open ends, making scoring complex.

### End of Hand Scoring

Remaining pips are totaled, rounded to nearest multiple of 5, divided by 5, and **subtracted** from that player's score.

### Game Target

61 points. If multiple players exceed 61, highest score wins. Ties require additional hands.

---

## Scoring Summary Table

|  Variation  |           In-Play Scoring          |    End-of-Hand Scoring    | Game Target |
|-------------|------------------------------------|---------------------------|-------------|
| Traditional | None                               | Opponents' pips           |      100/61 |
| All Fives   | Multiples of 5                     | Rounded pips              |     200-250 |
| All Sevens  | None                               | Opponents' minus own pips |         100 |
| Bergen      | Headers (2-3 pts) + domino (2 pts) | Winner: 2 points          |       10-15 |
| Blind       | None                               | Opponents' minus own pips |      100/61 |
| Five Up     | Multiples of 5 (÷5)                | Rounded pips (subtracted) |          61 |

---

## Spinner Summary

|  Variation  |   Spinner Rule    |
|-------------|-------------------|
| Traditional |       No spinners |
| All Fives   | First double only |
| All Sevens  |       No spinners |
| Bergen      |       No spinners |
| Blind       |       No spinners |
| Five Up     |       All doubles |

---

## Hand Size Summary (Double-6 Set)

|  Variation  | 2P  |  3P | 4P |
|-------------|-----|-----|----|
| Traditional |   7 | 6-7 |  6 |
|   All Fives |   9 |   7 |  5 |
|  All Sevens |   7 |   7 |  7 |
|      Bergen |   6 |   6 |  5 |
|       Blind |   8 |   7 |  6 |
|     Five Up | 5-7 |   5 |  5 |

---

## Sources

- [Pagat - Draw Dominoes](https://www.pagat.com/domino/line/draw.html)
- [Pagat - Bergen](https://www.pagat.com/domino/line/bergen.html)
- [Pagat - All Fives](https://www.pagat.com/domino/cross/all_fives.html)
- [Pagat - Five Up](https://www.pagat.com/domino/tree/five_up.html)
- [Pagat - Muggins](https://www.pagat.com/domino/line/muggins.html)
- [Pagat - Blind Hughie](https://www.pagat.com/tile/wdom/blind_hughie.html)
- [Pagat - Matador](https://www.pagat.com/tile/wdom/matador.html)
- [DominoRules.com - Bergen](https://www.dominorules.com/bergen)
- [DominoRules.com - Five Up](https://www.dominorules.com/five-up)
- [Domino-Games.com - All Threes](http://www.domino-games.com/domino-rules/allthrees-rules.html)
- [Wikipedia - Muggins](https://en.wikipedia.org/wiki/Muggins)
