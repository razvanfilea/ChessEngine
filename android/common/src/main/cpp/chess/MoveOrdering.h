#pragma once

#include "MoveGen.h"
#include "algorithm/Search.h"
#include "Thread.h"

class Board;

namespace MoveOrdering
{
	void sortMoves(const Thread &thread, const Board &board, MoveList &moveList) noexcept;

	void sortQMoves(MoveList &moveList) noexcept;

	Move getNextMove(MoveList &moveList) noexcept;
}
