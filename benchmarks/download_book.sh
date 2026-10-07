#!/usr/bin/env bash
set -e

# UHO (unbalanced human openings): every position favours one side, so far fewer draws than
# balanced books and more information per SPRT game pair. Stockfish's fishtest book.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BOOK_NAME="UHO_Lichess_4852_v1.epd"
BOOK_PATH="${SCRIPT_DIR}/${BOOK_NAME}"

if [ -f "$BOOK_PATH" ]; then
    echo "==> Opening book already exists at: $BOOK_PATH"
    exit 0
fi

echo "==> Downloading ${BOOK_NAME} opening book..."
curl -sSL "https://github.com/official-stockfish/books/raw/master/${BOOK_NAME}.zip" -o "${BOOK_PATH}.zip"
unzip -q -o "${BOOK_PATH}.zip" -d "${SCRIPT_DIR}"
rm -f "${BOOK_PATH}.zip"

echo "==> Successfully downloaded $(wc -l < "$BOOK_PATH") opening positions to: $BOOK_PATH"
