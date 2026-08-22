#pragma once

#include <array>

#include "Piece.h"

class Board;

namespace Zobrist
{
	u64 compute(const Board &board) noexcept;

	void xorPiece(u64 &key, Square square, Piece piece) noexcept;
	void flipSide(u64 &key) noexcept;
	void xorCastlingRights(u64 &key, CastlingRights rights) noexcept;
	void xorEnPassant(u64 &key, Square square) noexcept;
}
