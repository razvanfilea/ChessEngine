#!/usr/bin/env bash
# ==============================================================================
# CCRL Blitz Rating List Benchmark Tournament
#
# Emulates CCRL Blitz testing standards:
#   - Ponder: Off
#   - General book up to 12 moves (24 plies) using 8moves_v3.pgn (34,700 positions)
#   - Time control: Equivalent to 2'+1" on an Intel i7-4770K (default: 120+1 or scaled)
#   - Adjudication: Standard CCRL draw (move 40, 8 moves <= 10 cp) and resign (5 moves <= -600 cp)
#   - Tournament: Round-robin (all engines play each other equally)
#   - Hardware: Single thread per game, 64MB hash
# ==============================================================================
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
BIN_DIR="${SCRIPT_DIR}/bin"
PGN_DIR="${SCRIPT_DIR}/pgn"

mkdir -p "${BIN_DIR}" "${PGN_DIR}"

# 1. Configuration & CLI overrides
# Default TC is CCRL 2'+1" (120s + 1s). Pass custom TC as 1st arg if desired (e.g. 60+0.6 or 8+0.08).
TC="${1:-120+1}"
ROUNDS="${2:-50}"
CORES="${3:-12}"

BOOK_PATH="${SCRIPT_DIR}/8moves_v3.pgn"
if [ ! -f "${BOOK_PATH}" ]; then
    echo "==> Downloading 8moves_v3.pgn opening book (12-move depth)..."
    curl -sSL "https://raw.githubusercontent.com/official-stockfish/books/master/8moves_v3.pgn.zip" -o "${BOOK_PATH}.zip"
    unzip -q -o "${BOOK_PATH}.zip" -d "${SCRIPT_DIR}"
    rm -f "${BOOK_PATH}.zip"
fi

# Optional Syzygy tablebases (if set in environment EGTB_PATH)
EGTB_OPT=""
if [ -n "${EGTB_PATH}" ] && [ -d "${EGTB_PATH}" ]; then
    echo "==> Syzygy EGTB enabled at: ${EGTB_PATH}"
    EGTB_OPT="option.SyzygyPath=${EGTB_PATH}"
fi

# 2. Compile current lucky_chess dev binary
echo "==> Compiling lucky_chess release binary..."
cargo build --release --manifest-path "${REPO_ROOT}/Cargo.toml" --bin lucky_chess
cp "${REPO_ROOT}/target/release/lucky_chess" "${BIN_DIR}/lucky_dev"

TIMESTAMP="$(date +%Y%m%d_%H%M%S)"
PGN_OUT="${PGN_DIR}/ccrl_blitz_${TIMESTAMP}.pgn"

echo "=========================================================="
echo " CCRL Blitz Benchmark Tournament"
echo " Time Control : ${TC}"
echo " Tournament   : Round-Robin (${ROUNDS} rounds, paired openings)"
echo " Concurrency  : ${CORES} concurrent games"
echo " Opening Book : $(basename "${BOOK_PATH}") (plies=24 / 12 moves max)"
echo " Engines      :"
echo "   - lucky_dev"
echo "   - Oxide_3495"
echo "   - Bitbit_3410"
echo "   - Princhess_3357"
echo "   - Monolith_3261"
echo "=========================================================="

fastchess \
  -tournament roundrobin \
  -engine cmd="${BIN_DIR}/lucky_dev" name=lucky_dev \
  -engine cmd="${BIN_DIR}/OxideV2.0.0_ELO_3495" name=Oxide_3495 \
  -engine cmd="${BIN_DIR}/bitbit-1.7_ELO_3410" name=Bitbit_3410 \
  -engine cmd="${BIN_DIR}/princhess_ELO_3357" name=Princhess_3357 \
  -engine cmd="${BIN_DIR}/Monolith-linux-x86-64-pext_ELO_3261" name=Monolith_3261 \
  -each tc="${TC}" option.Hash=64 option.Threads=1 option.Ponder=false ${EGTB_OPT} \
  -rounds "${ROUNDS}" \
  -repeat \
  -games 2 \
  -concurrency "${CORES}" \
  -draw movenumber=40 movecount=8 score=10 \
  -resign movecount=5 score=600 \
  -openings file="${BOOK_PATH}" format=pgn order=random plies=24 \
  -pgnout file="${PGN_OUT}" notation=san

# Clean up temporary fastchess config
rm -f "${REPO_ROOT}/config.json" "${SCRIPT_DIR}/config.json"

echo ""
echo "==> Tournament finished. PGN saved to: ${PGN_OUT}"
if [ -f "${SCRIPT_DIR}/analyze_pgn.py" ]; then
    python3 "${SCRIPT_DIR}/analyze_pgn.py" "${PGN_OUT}"
fi
