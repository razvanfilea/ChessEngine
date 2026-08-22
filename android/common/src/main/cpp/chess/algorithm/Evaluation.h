#pragma once

#include <string>

#include "../Board.h"

class Evaluation final
{
public:
	static constexpr i16 TEMPO_BONUS = 20;

	static i32 value(const Board &board) noexcept;
	static i32 invertedValue(const Board &board) noexcept;
	static std::string traceValue(const Board &board);

	static constexpr i16 getNpmValue(const PieceType type) noexcept
	{
		constexpr i16 PieceValue[] = { 0, 0, 781, 825, 1276, 2538, 0 };

		return PieceValue[type];
	}

	static constexpr i16 getPieceValue(const PieceType type) noexcept
	{
		constexpr i16 PieceValue[] = { 0, 128, 781, 825, 1276, 2538, 0 };

		return PieceValue[type];
	}
};
