#pragma once

#include <algorithm>

#include "Move.h"
#include "Board.h"

class MoveList
{
	void generateMoves() noexcept;

public:
	explicit MoveList(Board &board)
		: _board(board), _end(begin())
	{
		generateMoves();
	}

	constexpr Move *begin() noexcept { return _moveList; }

	[[nodiscard]] constexpr const Move *begin() const noexcept { return _moveList; }

	constexpr Move *end() noexcept { return _end; }

	[[nodiscard]] constexpr const Move *end() const noexcept { return _end; }

	constexpr Move &front() noexcept { return *begin(); }

	[[nodiscard]] constexpr const Move &front() const noexcept { return *begin(); }

	constexpr Move &back() noexcept { return *std::prev(end()); }

	[[nodiscard]] constexpr const Move &back() const noexcept { return *std::prev(end()); }

	constexpr void popBack() noexcept { --_end; }

	[[nodiscard]] constexpr usize size() const noexcept { return _end - _moveList; }

	[[nodiscard]] constexpr bool empty() const noexcept { return size() == 0u; }

	[[nodiscard]] constexpr bool contains(const Move &move) const noexcept
	{
		for (auto &&m : *this)
			if (m == move)
				return true;

		return false;
	}

	template <class... Args>
	constexpr void emplace_back(Args &&...args) noexcept
	{
		*_end++ = Move(std::forward<Args>(args)...);
	}

	void keepLegalMoves() noexcept
	{
		auto it = begin();
		while (it != end())
		{
			if (!_board.isMoveLegal(*it))
			{
				*it = back();
				popBack();
			} else
				++it;
		}
	}

private:
	Board &_board;
	Move _moveList[MAX_MOVES];
	Move *_end;
};

inline bool moveExists(Board &board, const Move &move) noexcept
{
	const MoveList moveList(board);

	for (const Move &m : moveList)
		if (m == move)
			return board.isMoveLegal(m);

	return false;
}

inline Move parseMove(Board &board, const std::string &str)
{
	if (str[1] > '8' || str[1] < '1'
		|| str[3] > '8' || str[3] < '1'
		|| str[0] > 'h' || str[0] < 'a'
		|| str[2] > 'h' || str[2] < 'a')
		return {};

	const Square from = toSquare(u8(str[0] - 'a'), u8(str[1] - '1'));
	const Square to = toSquare(u8(str[2] - 'a'), u8(str[3] - '1'));

	const MoveList moveList(board);

	for (const Move &move : moveList)
	{
		if (move.from() == from && move.to() == to)
		{
			if (move.flags().promotion())
			{
				const PieceType promoted = move.promotedPiece();
				if ((promoted == KNIGHT && str[4] == 'k')
					|| (promoted == BISHOP && str[4] == 'b')
					|| (promoted == ROOK && str[4] == 'r')
					|| (promoted == ROOK && str[4] == 'q'))
					return move;

				continue;
			}
			return move;
		}
	}

	return {};
}
