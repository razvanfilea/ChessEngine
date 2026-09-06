#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
BIN_DIR="${SCRIPT_DIR}/bin"
PGN_DIR="${SCRIPT_DIR}/pgn"

mkdir -p "${BIN_DIR}" "${PGN_DIR}"
chmod +x "${BIN_DIR}"/* 2>/dev/null || true

TC="${1:-5+0.1}"
ROUNDS="${2:-100}"
CORES="${3:-12}"

# 1. Ensure opening book exists
"${SCRIPT_DIR}/download_book.sh"

# 2. Compile current dev binary
echo "==> Compiling dev binary in release mode..."
cargo build --release --manifest-path "${REPO_ROOT}/Cargo.toml" --bin lucky_chess
cp "${REPO_ROOT}/target/release/lucky_chess" "${BIN_DIR}/lucky_dev"

TIMESTAMP="$(date +%Y%m%d_%H%M%S)"
PGN_OUT="${PGN_DIR}/gauntlet_${TIMESTAMP}.pgn"

echo "=========================================================="
echo " Running Fast Gauntlet Benchmark"
echo " Time Control : ${TC} | Rounds: ${ROUNDS} | Concurrency: ${CORES}"
echo " Opponents    : Pawn 3557, Ursus 3509, Oxide 3495, Bitbit 3410"
echo "=========================================================="

fastchess \
  -tournament gauntlet \
  -engine cmd="${BIN_DIR}/lucky_dev" name=lucky_dev \
  -engine cmd="${BIN_DIR}/pawn-v4.0-3557" name=Pawn_3557 \
  -engine cmd="${BIN_DIR}/ursus-ELO_3509" name=Ursus_3509 \
  -engine cmd="${BIN_DIR}/OxideV2.0.0_ELO_3495" name=Oxide_3495 \
  -engine cmd="${BIN_DIR}/sirius-9.0_ELO_3528" name=Sirius_3528 \
  -each tc="${TC}" option.Hash=64 option.Threads=1 \
  -rounds "${ROUNDS}" \
  -repeat \
  -games 2 \
  -concurrency "${CORES}" \
  -draw movenumber=40 movecount=8 score=10 \
  -resign movecount=5 score=600 \
  -openings file="${SCRIPT_DIR}/noob_3moves.epd" format=epd order=random \
  -sprt elo0=0 elo1=50 alpha=0.05 beta=0.05 \
  -pgnout file="${PGN_OUT}" notation=san

# Clean up temporary fastchess config
rm -f "${REPO_ROOT}/config.json" "${SCRIPT_DIR}/config.json"

echo ""
echo "==> Gauntlet finished. PGN saved to: ${PGN_OUT}"
