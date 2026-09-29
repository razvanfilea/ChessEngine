# LuckyChess

[![Rust](https://img.shields.io/badge/Rust-2024_Edition-orange.svg)](https://www.rust-lang.org/)
[![License: GPL-3.0](https://img.shields.io/badge/License-GPLv3-blue.svg)](LICENSE.txt)
![Estimated Rating](https://img.shields.io/badge/CCRL%20Blitz-~3,570%20--%203,590%20Elo-brightgreen.svg)
[![WASM](https://img.shields.io/badge/WebAssembly-Supported-purple.svg)](chess_web/)
[![CI](https://github.com/razvanfilea/ChessEngine/actions/workflows/ci.yml/badge.svg)](https://github.com/razvanfilea/ChessEngine/actions/workflows/ci.yml)

A strong, modern UCI chess engine written in Rust featuring an embedded NNUE evaluation network, cross-platform support (CLI, Web/WASM, Android).

---

## Table of Contents
- [Building](#building)
- [Web & Docker](#web--docker)
- [Implemented Algorithms & Techniques](#implemented-algorithms--techniques)
- [UCI Protocol & Options](#uci-protocol--options)
- [Benchmarks & Testing](#benchmarks--testing)
- [Workspace Architecture](#workspace-architecture)
- [Acknowledgments & Credits](#acknowledgments--credits)
- [License](#license)

---

## Building

### Prerequisites
- [Rust toolchain](https://rustup.rs/) (Rust 1.98+ / 2024 edition compatible).
- [Git LFS](https://git-lfs.com/) — the NNUE network weights (`chess_engine/src/nnue/*.bin`) are stored with Git LFS. Run `git lfs pull` after cloning if your git client doesn't fetch LFS objects automatically.

### Compiling
Build the optimized release executable:

```bash
cargo build --release --bin lucky_chess
```

For maximum performance on your current machine:

```bash
RUSTFLAGS="-C target-cpu=native" cargo build --release --bin lucky_chess
```

The resulting binary will be at `target/release/lucky_chess` (or `lucky_chess.exe` on Windows).

---

## Web & Docker

LuckyChess can run entirely in the browser via WebAssembly using Web Workers.

### Running with Docker
```bash
docker build -t lucky-chess .
docker run -p 8080:8080 lucky-chess
```
Then visit `http://localhost:8080` to play against the engine in your browser.

### Building WASM Manually
```bash
cargo build-wasm
```

---

## Implemented Algorithms & Techniques

**Move Generation**
- Bitboards with PEXT slider attacks when BMI2 is available, magic bitboards otherwise (tables generated at build time)
- Staged move picker: TT move → captures → quiets → bad captures (evasions only when in check)
- Pseudo-legal generation with a legality filter; `gives_check` from a per-node `CheckInfo`
- Static Exchange Evaluation (SEE) for move ordering and pruning

**Search**
- Alpha-Beta Pruning (Negamax framework)
- Principal Variation Search (PVS)
- Iterative Deepening with Aspiration Windows
- Transposition Table with aging and depth-preferred replacement, storing static eval, prefetched after each move
- Internal Iterative Reductions (IIR)
- Quiescence Search

**Reductions & Pruning**
- Late Move Reductions (LMR) with history-based adjustment
- Late Move Pruning (LMP)
- Null Move Pruning (NMP) with eval-scaled reduction
- Reverse Futility Pruning (RFP) & Futility Pruning
- SEE Pruning for captures and quiets
- History Pruning (linear depth-scaled margin)
- Delta Pruning & Global Delta Pruning (in Quiescence Search)

**Move Ordering**
- TT move (scored highest)
- Good captures (SEE + MVV-LVA)
- Killer Move Heuristic (2 killer moves per ply)
- History Heuristic with gravity updates (clamped to ±10,000)
- Multi-layer Continuation History (1-ply and 2-ply reads)
- Bad captures deferred after quiets

**Evaluation**
- **NNUE**: `(768×8 → 1536)x2 → 1×8` architecture, SCReLU activation, QA = 255 / QB = 64
  - 8 king buckets on the input side, mirrored horizontally (king on files e–h flips the board)
  - 8 output buckets selected by piece count
- Perspective network (vertically mirrored for black)
- Lazy accumulator stack: incremental updates from the parent ply, with Finny-table refreshes when a king changes bucket
- Fused SIMD update + SCReLU dot-product kernels via `fearless_simd`
- Win-probability scaled evaluation (~400 units/pawn)
- Trained on 20B positions of Leela-derived data ([linrock's test77/test79 sets](https://huggingface.co/datasets/linrock/bullet-training-data)) using the [`bullet`](https://github.com/jw1912/bullet) framework

**Time Management**
- Soft and hard time limits; best-move stability decides whether to start another iteration
- Supports `wtime`/`btime`/`winc`/`binc`/`movestogo`, `movetime`, `depth`, `nodes` and `infinite`
- Configurable `Move Overhead` for GUI and network lag

---

## UCI Protocol & Options

### Standard UCI Commands
`uci`, `isready`, `setoption`, `ucinewgame`, `position`, `go`, `stop`, `quit`.

### Additional Commands
- `eval` — static NNUE evaluation of the current position
- `d` / `display` — print the current board
- `perft <depth>` / `go perft <depth>` — move generation/validation performance test

The `lucky_chess` binary also accepts these, on stdin or as arguments (e.g. `lucky_chess bench 12`):
- `bench [depth]` — deterministic search over a fixed set of positions (default depth 10, 16 MB hash)
- `perft-suite [depth]` — perft over a suite of reference positions

### Configurable Options
| Option | Type | Default | Range | Description |
|---|---|---|---|---|
| `Hash` | spin | `64` | 1 – 1024 MB | Transposition table memory size |
| `ClearHash` | button | — | — | Clears the transposition table |
| `Move Overhead` | spin | `10` | 0 – 5000 ms | Time buffer for communication / GUI lag |
| `Threads` | spin | `1` | 1 – 1 | Single-threaded search (accepted for GUI compatibility) |

---

## Benchmarks & Testing

### Tests
`cargo test --all` runs the suite in `chess_engine/tests/`: perft, per-piece move generation, legality, make/undo,
FEN, SEE, Zobrist hashing, draw detection, time management, and NNUE accumulator-stack vs from-scratch evaluation.

### SPRT
`benchmarks/sprt_self_play.sh [elo0] [elo1] [tc]` builds the current tree and plays it against a saved baseline
(`benchmarks/save_baseline.sh`) with [`fastchess`](https://github.com/Disservin/fastchess). Defaults: `8+0.08`,
bounds `[0, 5]`.

### Gauntlet
`benchmarks/gauntlet_ccrl_blitz.sh [tc] [rounds] [concurrency]` runs a CCRL Blitz-style gauntlet (8moves_v3 book,
CCRL adjudication, 1 thread) against Pawn 4.0, Prune 4.0.1, Oxide 3.0 and Akimbo 1.0.0; each round is 8 games.

Latest result (v5 net, 160 games per engine, before Akimbo replaced Ursus):

| Rank | Engine | Elo | Score |
|---|---|---|---|
| 1 | **LuckyChess (v5)** | **+50.3 ± 29.0** | **57.2%** |
| 2 | Pawn 4.0 (3557) | +43.7 ± 31.0 | 56.2% |
| 3 | Sirius 9.0 (3528) | +2.2 ± 34.4 | 50.3% |
| 4 | Oxide 3.0 (3530) | −6.5 ± 24.5 | 49.1% |
| 5 | Ursus (3509) | −91.1 ± 31.7 | 37.2% |

Against a field averaging ~3,530 CCRL Blitz this is roughly 3,580; with ±29 Elo of noise, the badge gives a range.

---

## Workspace Architecture

```text
├── chess_core/       # Core types: bitboards, squares, pieces, moves, castling rights
├── chess_engine/     # Board state, move generation, search, NNUE evaluation, time management, UCI protocol, perft
├── chess_cli/        # Native CLI binary executable (lucky_chess)
├── chess_web/        # C-FFI / WebAssembly cdylib & browser frontend
├── chess_android/    # JNI bindings and Android / Wear OS companion app
├── nnue_trainer/     # NNUE training pipeline and utilities
└── benchmarks/       # SPRT testing scripts, opening books, gauntlet suites
```

---

## Acknowledgments & Credits

While the codebase is original, LuckyChess stands on the shoulders of the open-source chess programming community:

- **[Stockfish](https://github.com/official-stockfish/Stockfish)**
- **[Alexandria](https://github.com/mhouppin/alexandria)**
- **[jw1912](https://github.com/jw1912)** — creator of the [`bullet`](https://github.com/jw1912/bullet) training framework
- **[linrock](https://huggingface.co/linrock)** — Leela-derived NNUE training data
- **[Chess Programming Wiki](https://www.chessprogramming.org/)** — invaluable resource for chess algorithms, magic bitboards, and search techniques
- **Tools & Libraries**:
  - `uci-parser` for UCI command parsing
  - `fearless_simd` for portable SIMD acceleration
  - `fastchess` for automated SPRT testing

---

## Development Note & AI Usage
LuckyChess is an original engine. In the interest of transparency within the chess programming community: AI assistants (LLMs) were used during development for tests and fixtures, profiling and benchmark analysis, and code review. The engine's architecture, design decisions and training runs are the author's, and every engine change was validated by SPRT.

---

## License

LuckyChess is free and open-source software licensed under the [GNU General Public License v3.0](LICENSE.txt).
