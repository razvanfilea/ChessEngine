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

echo "==> Running gauntlet..."
fastchess \
  -tournament gauntlet \
  -engine cmd="${BIN_DIR}/lucky_dev" name=lucky_dev \
  -engine cmd="${BIN_DIR}/lucky_base" name=lucky_base \
  -engine cmd="${BIN_DIR}/cheers-v1.0.0-x86_64-linux-gnu-avx2_ELO_3033" name=Cheers_3033 \
  -engine cmd="${BIN_DIR}/Monolith-linux-x86-64-pext_ELO_3261" name=Monolith_3261 \
  -each tc=5+0.1 \
  -rounds 100 \
  -repeat \
  -games 2 \
  -concurrency 12 \
  -openings file="${SCRIPT_DIR}/noob_3moves.epd" format=epd order=random \
  -sprt elo0=0 elo1=50 alpha=0.05 beta=0.05 \
  -pgnout file="${PGN_DIR}/gauntlet_nnue_v2.pgn"
