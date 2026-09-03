#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
BIN_DIR="${SCRIPT_DIR}/bin"
PGN_DIR="${SCRIPT_DIR}/pgn"

mkdir -p "${BIN_DIR}" "${PGN_DIR}"

echo "==> Compiling dev binary in release mode..."
cargo build --release --manifest-path "${REPO_ROOT}/Cargo.toml" --bin lucky_chess
cp "${REPO_ROOT}/target/release/lucky_chess" "${BIN_DIR}/lucky_dev"

# -engine cmd="${BIN_DIR}/lucky_base" name=lucky_base \
echo "==> Running gauntlet..."
fastchess \
  -tournament gauntlet \
  -engine cmd="${BIN_DIR}/lucky_dev" name=lucky_dev \
  -engine cmd="${BIN_DIR}/Monolith-linux-x86-64-pext_ELO_3261" name=Monolith_3261 \
  -engine cmd="${BIN_DIR}/princhess_ELO_3357" name=Princhess_3357 \
  -engine cmd="${BIN_DIR}/bitbit-1.7_ELO_3410" name=Bitbit_3410 \
  -each tc=5+0.1 \
  -rounds 100 \
  -repeat \
  -games 2 \
  -concurrency 12 \
  -openings file="${SCRIPT_DIR}/noob_3moves.epd" format=epd order=random \
  -sprt elo0=0 elo1=50 alpha=0.05 beta=0.05 \
  -pgnout file="${PGN_DIR}/gauntlet_nnue_v3.pgn"
