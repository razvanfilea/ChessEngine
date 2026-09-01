#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BOOK_PATH="${SCRIPT_DIR}/noob_3moves.epd"

if [ -f "$BOOK_PATH" ]; then
    echo "==> Opening book already exists at: $BOOK_PATH"
    exit 0
fi

echo "==> Downloading noob_3moves.epd opening book..."
curl -sSL "https://github.com/official-stockfish/books/raw/master/noob_3moves.epd.zip" -o "${SCRIPT_DIR}/noob_3moves.epd.zip"
unzip -q -o "${SCRIPT_DIR}/noob_3moves.epd.zip" -d "${SCRIPT_DIR}"
rm -f "${SCRIPT_DIR}/noob_3moves.epd.zip"

echo "==> Successfully downloaded $(wc -l < "$BOOK_PATH") opening positions to: $BOOK_PATH"
