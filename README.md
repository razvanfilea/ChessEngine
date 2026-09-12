# Lucky Chess

A chess engine written in Rust.

## Implemented Algorithms & Techniques

**Search**
- Alpha-Beta Pruning (Negamax framework)
- Principal Variation Search (PVS)
- Iterative Deepening
- Aspiration Windows
- Quiescence Search

**Reductions & Pruning**
- Late Move Reductions (LMR) with history-based adjustment
- Late Move Pruning (LMP)
- Null Move Pruning (NMP) with eval-scaled reduction
- Reverse Futility Pruning (RFP)
- Futility Pruning
- SEE Pruning for captures and quiets
- History Pruning
- Delta Pruning (in Quiescence Search)

**Move Ordering**
- TT move
- MVV-LVA with SEE for capture ordering
- Killer Move Heuristic
- History Heuristic (gravity updates)
- Continuation History (multi-layer: 1-ply and 2-ply)

**Evaluation**
- NNUE (768 → 1536)x2 → 8 architecture with output buckets by piece count
- Perspective network (vertically flipped for black) with incremental accumulator updates
- Trained on [Stockfish-generated data](https://huggingface.co/datasets/linrock/bullet-training-data)

## UCI Protocol
Standard UCI commands: `uci`, `isready`, `setoption`, `ucinewgame`, `position`, `go`, `stop`, `quit`.

Additional commands:
- `bench [depth]` — deterministic search over a fixed set of positions
- `go perft <depth>` — move generation/validation performance test

UCI options:
- `Hash` (1–1024 MB, default 64) — transposition table size
- `ClearHash` — clear the transposition table
- `Move Overhead` (0–5000 ms, default 10) — time buffer for communication delay
