# Lucky Chess

A chess engine written in Rust.

## Implemented Algorithms & Techniques

**Search & Pruning**
- Alpha-Beta Pruning (Negamax framework)
- Principal Variation Search (PVS)
- Quiescence Search
- Aspiration Windows

**Reductions & Forward Pruning**
- Late Move Reductions (LMR)
- Null Move Pruning (NMP)
- Futility Pruning
- Reverse Futility Pruning (RFP) / Static Null Move Pruning
- Delta Pruning (in Quiescence Search)

**Move Ordering & Heuristics**
- Transposition Table (TT)
- Killer Move Heuristic
- History Heuristic

## UCI Protocol
Supports standard Universal Chess Interface (UCI) commands (`uci`, `isready`, `position`, `go`, `stop`) for use with modern chess GUIs.
