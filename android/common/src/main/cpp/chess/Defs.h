#pragma once

#include <cstdint>
#include <cstddef>
#include <cassert>
#include <algorithm>

#ifdef NDEBUG
    #ifdef _MSC_VER
        #define force_inline inline __forceinline
    #else
        #define force_inline inline __attribute__((always_inline))
    #endif
#else
    #define force_inline inline
#endif

using i8 = std::int8_t;
using i16 = std::int16_t;
using i32 = std::int32_t;
using i64 = std::int64_t;

using u8 = std::uint8_t;
using u16 = std::uint16_t;
using u32 = std::uint32_t;
using u64 = std::uint64_t;
using usize = std::size_t;

enum Color : bool
{
	BLACK = false,
	WHITE = true
};

enum
{
	COLOR_NB = 2
};

force_inline constexpr Color operator~(const Color c) noexcept
{
	return Color(c ^ WHITE); // Toggle color
}

enum Dir : i16
{
	NORTH,
	SOUTH,
	EAST,
	WEST,
	NORTH_EAST,
	NORTH_WEST,
	SOUTH_EAST,
	SOUTH_WEST
};

enum CastlingRights : u8
{
	CASTLE_NONE = 0,

	CASTLE_BLACK_KING = 0b0001,
	CASTLE_BLACK_QUEEN = 0b0010,
	CASTLE_BLACK_BOTH = CASTLE_BLACK_KING | CASTLE_BLACK_QUEEN,
	CASTLED_BLACK = 0b0100,

	CASTLE_WHITE_KING = 0b0'1000,
	CASTLE_WHITE_QUEEN = 0b1'0000,
	CASTLE_WHITE_BOTH = CASTLE_WHITE_KING | CASTLE_WHITE_QUEEN,
	CASTLED_WHITE = 0b10'0000
};

enum Phase : u8
{
	END_GAME_PHASE,
	MIDDLE_GAME_PHASE = 128
};

enum Value : i16
{
	VALUE_NONE = 32700,
	VALUE_MAX = 32001,
	VALUE_MIN = -VALUE_MAX,
	VALUE_MATE = VALUE_MAX - 1,

	MAX_MOVES = 256,
	MAX_DEPTH = 64,

	VALUE_MATE_MAX_DEPTH = VALUE_MATE - MAX_DEPTH
};

constexpr bool isMateValue(const Value value) noexcept
{
	const int absValue = value < 0 ? -value : value;
	return (absValue + MAX_DEPTH + 1) > VALUE_MAX;
}

enum PieceType : u8
{
	NO_PIECE_TYPE = 0,
	PAWN = 1,
	KNIGHT = 2,
	BISHOP = 3,
	ROOK = 4,
	QUEEN = 5,
	KING = 6,

	PIECE_TYPE_NB = 7
};

enum Square : u8
{
	SQ_A1, SQ_B1, SQ_C1, SQ_D1, SQ_E1, SQ_F1, SQ_G1, SQ_H1,
	SQ_A2, SQ_B2, SQ_C2, SQ_D2, SQ_E2, SQ_F2, SQ_G2, SQ_H2,
	SQ_A3, SQ_B3, SQ_C3, SQ_D3, SQ_E3, SQ_F3, SQ_G3, SQ_H3,
	SQ_A4, SQ_B4, SQ_C4, SQ_D4, SQ_E4, SQ_F4, SQ_G4, SQ_H4,
	SQ_A5, SQ_B5, SQ_C5, SQ_D5, SQ_E5, SQ_F5, SQ_G5, SQ_H5,
	SQ_A6, SQ_B6, SQ_C6, SQ_D6, SQ_E6, SQ_F6, SQ_G6, SQ_H6,
	SQ_A7, SQ_B7, SQ_C7, SQ_D7, SQ_E7, SQ_F7, SQ_G7, SQ_H7,
	SQ_A8, SQ_B8, SQ_C8, SQ_D8, SQ_E8, SQ_F8, SQ_G8, SQ_H8,

	SQ_NONE = 64,
	SQUARE_NB = 64
};

force_inline constexpr Square toSquare(const u8 square) noexcept
{
	return static_cast<Square>(square);
}

force_inline constexpr Square &operator++(Square &square) noexcept
{
	return square = static_cast<Square>(square + 1u);
}

force_inline constexpr Square toSquare(const u8 x, const u8 y) noexcept
{
	return static_cast<Square>((y << 3u) + x);
}

template <Dir D>
constexpr Square shift(const Square square) noexcept
{
	i8 s{};
	if constexpr (D == NORTH)
		s = 8;
	else if constexpr (D == SOUTH)
		s = -8;
	else if constexpr (D == EAST)
		s = 1;
	else if constexpr (D == WEST)
		s = -1;

	else if constexpr (D == NORTH_EAST)
		s = 8 + 1;
	else if constexpr (D == NORTH_WEST)
		s = 8 - 1;
	else if constexpr (D == SOUTH_EAST)
		s = -8 + 1;
	else if constexpr (D == SOUTH_WEST)
		s = -8 - 1;

	return toSquare(square + s);
}

force_inline constexpr Square capturedEnPassantSq(const Color colorToMove, const Square enPassantSq) noexcept
{
	return (colorToMove == WHITE) ? shift<SOUTH>(enPassantSq) : shift<NORTH>(enPassantSq);
}

force_inline constexpr Square shiftToKingRank(const Color colorToMove, const Square square) noexcept
{
	if (colorToMove == BLACK)
		return toSquare(square + 56u);
	return square;
}

constexpr u8 rankOf(const u8 square) noexcept { return u8(square >> 3u); }

constexpr u8 fileOf(const u8 square) noexcept { return u8(square & 7u); }

constexpr u8 distanceToFileEdge(const Square square) noexcept
{
	const auto file = fileOf(square);
	return std::min<u8>(file, 7u - file);
}

constexpr u8 distanceToRankEdge(const Square square) noexcept
{
	const auto rank = rankOf(square);
	return std::min<u8>(rank, 7u - rank);
}


class Score final
{
public:
	Score() = default;

	constexpr Score(const i16 mg, const i16 eg) noexcept
		: mg(mg), eg(eg) {}

	constexpr Score &operator=(const i16 rhs) noexcept
	{
		mg = rhs;
		eg = rhs;
		return *this;
	}

	constexpr void operator+=(const Score &rhs) noexcept
	{
		mg += rhs.mg;
		eg += rhs.eg;
	}

	constexpr void operator-=(const Score &rhs) noexcept
	{
		mg -= rhs.mg;
		eg -= rhs.eg;
	}

	constexpr void operator+=(const i16 rhs) noexcept
	{
		mg += rhs;
		eg += rhs;
	}

	constexpr void operator-=(const i16 rhs) noexcept
	{
		mg -= rhs;
		eg -= rhs;
	}

	constexpr Score operator+(const Score &rhs) const noexcept
	{
		Score lhs = *this;
		lhs.mg += rhs.mg;
		lhs.eg += rhs.eg;
		return lhs;
	}

	constexpr Score operator-(const Score &rhs) const noexcept
	{
		Score lhs = *this;
		lhs.mg -= rhs.mg;
		lhs.eg -= rhs.eg;
		return lhs;
	}

	constexpr Score operator*(const i16 rhs) const noexcept
	{
		Score lhs = *this;
		lhs.mg *= rhs;
		lhs.eg *= rhs;
		return lhs;
	}

	i16 mg{}, eg{};
};
