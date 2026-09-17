# LuckyChess

[![Rust](https://img.shields.io/badge/Rust-2024_Edition-orange.svg)](https://www.rust-lang.org/)
[![License: GPL-3.0](https://img.shields.io/badge/License-GPLv3-blue.svg)](LICENSE.txt)
[![Estimated Rating](https://img.shields.io/badge/CCRL%20Blitz-~3560%20Elo-brightgreen.svg)](HANDOFF.md)
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
- [Rust toolchain](https://rustup.rs/) (Rust 1.85+ / 2024 edition compatible).

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

**Search**
- Alpha-Beta Pruning (Negamax framework)
- Principal Variation Search (PVS)
- Iterative Deepening with Aspiration Windows
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
- **NNUE**: `(768 → 1536)x2 → 8` architecture with output buckets by piece count
- Perspective network (vertically mirrored for black) with incremental accumulator updates
- Win-probability scaled evaluation (~400 units/pawn)
- Trained on [Stockfish-generated data](https://huggingface.co/datasets/linrock/bullet-training-data) using the [`bullet`](https://github.com/jw1912/bullet) framework

---

## UCI Protocol & Options

### Standard UCI Commands
`uci`, `isready`, `setoption`, `ucinewgame`, `position`, `go`, `stop`, `quit`.

### Additional Commands
- `bench [depth]` — deterministic search over a fixed set of positions
- `go perft <depth>` — move generation/validation performance test

### Configurable Options
| Option | Type | Default | Range | Description |
|---|---|---|---|---|
| `Hash` | spin | `64` | 1 – 1024 MB | Transposition table memory size |
| `ClearHash` | button | — | — | Clears the transposition table |
| `Move Overhead` | spin | `10` | 0 – 5000 ms | Time buffer for communication / GUI lag |

---

## Benchmarks & Testing

- **CCRL Blitz Benchmark**: **~3,561 Elo** (±45) in 400-game round-robin testing (120s + 1s, 64MB hash).
- **SPRT & Gauntlet Scripts**: Located under `benchmarks/` using [fastchess](https://github.com/Disservin/fastchess):
  ```bash
  # Run CCRL blitz gauntlet tournament against reference engines
  ./benchmarks/gauntlet_ccrl_blitz.sh

  # Run SPRT self-play testing
  ./benchmarks/sprt_self_play.sh
  ```

---

## Workspace Architecture

```text
├── chess_core/       # Core types, bitboards, move generator, board state, perft
├── chess_engine/     # Search, NNUE evaluation, time management, UCI protocol
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
- **[Chess Programming Wiki](https://www.chessprogramming.org/)** — invaluable resource for chess algorithms, magic bitboards, and search techniques
- **Tools & Libraries**:
  - `uci-parser` for UCI command parsing
  - `fearless_simd` for portable SIMD acceleration
  - `fastchess` for automated SPRT testing

---

## Development Note & AI Usage
LuckyChess is an original engine. In the interest of transparency within the chess programming community, AI assistance (LLMs) was restricted exclusively to authoring unit tests and test fixtures. All algorithmic architecture — including move generation, search pruning and ordering heuristics, and the NNUE pipeline — was conceived and authored entirely by hand.

---

## License

LuckyChess is free and open-source software licensed under the [GNU General Public License v3.0](LICENSE.txt).
