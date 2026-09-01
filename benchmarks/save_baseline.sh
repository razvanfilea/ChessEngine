#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
BIN_DIR="${SCRIPT_DIR}/bin"

mkdir -p "${BIN_DIR}"

echo "==> Compiling baseline binary in release mode..."
cargo build --release --manifest-path "${REPO_ROOT}/Cargo.toml" --bin lucky_chess

cp "${REPO_ROOT}/target/release/lucky_chess" "${BIN_DIR}/lucky_base"

COMMIT_HASH="$(git -C "${REPO_ROOT}" rev-parse --short HEAD 2>/dev/null || echo "unknown")"
COMMIT_MSG="$(git -C "${REPO_ROOT}" log -1 --pretty=%B 2>/dev/null | head -n 1 || echo "unknown")"
DATE="$(date)"

cat <<EOF > "${BIN_DIR}/lucky_base.info"
Saved: ${DATE}
Commit: ${COMMIT_HASH}
Message: ${COMMIT_MSG}
EOF

echo "==> Baseline saved to ${BIN_DIR}/lucky_base"
echo "    Commit: [${COMMIT_HASH}] ${COMMIT_MSG}"
