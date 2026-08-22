#include "Zobrist.h"

#include <array>

#include "Board.h"

class RandomGenerator
{
public:
	/*
	 * Generate random numbers based on this paper: http://vigna.di.unimi.it/ftp/papers/xorshift.pdf
	 */
	constexpr u64 random64() noexcept
	{
		_seed ^= _seed >> 12u;
		_seed ^= _seed << 25u;
		_seed ^= _seed >> 27u;

		return _seed * 2685821657736338717ull;
	}

	template <usize N>
	constexpr auto randomArray() noexcept
	{
		std::array<u64, N> array{};

		for (auto &item : array)
			item = random64();

		return array;
	}

private:
	u64 _seed = 1070372ull;
};

namespace Zobrist
{
	constinit RandomGenerator Generator;

	static const auto PiecesKeys = []
	{
		std::array<std::array<std::array<u64, COLOR_NB>, PIECE_TYPE_NB>, SQUARE_NB> array{};

		for (auto &sq : array)
			for (auto &piece : sq)
				piece = Generator.randomArray<COLOR_NB>();

		return array;
	}();
	static const u64 SideKey{ Generator.random64() };
	static const auto CastlingRightsKeys{ Generator.randomArray<4>() };
	static const auto EnPassantKeys{ Generator.randomArray<8>() };

	u64 compute(const Board &board) noexcept
	{
		u64 hash{};

		for (u8 sq{}; sq < SQUARE_NB; ++sq)
			if (const Piece &piece = board.getSquare(toSquare(sq)); piece)
				hash ^= PiecesKeys[sq][piece.type()][piece.color()];

		xorCastlingRights(hash, static_cast<CastlingRights>(board.state.castlingRights));
		xorEnPassant(hash, board.getEnPassantSq());

		if (board.colorToMove)
			flipSide(hash);

		return hash;
	}

	void xorPiece(u64 &key, const Square square, const Piece piece) noexcept
	{
		key ^= PiecesKeys[square][piece.type()][piece.color()];
	}

	void flipSide(u64 &key) noexcept
	{
		key ^= SideKey;
	}

	void xorCastlingRights(u64 &key, const CastlingRights rights) noexcept
	{
		if (rights & CASTLE_WHITE_KING)
			key ^= CastlingRightsKeys[0];
		if (rights & CASTLE_WHITE_QUEEN)
			key ^= CastlingRightsKeys[1];

		if (rights & CASTLE_BLACK_KING)
			key ^= CastlingRightsKeys[2];
		if (rights & CASTLE_BLACK_QUEEN)
			key ^= CastlingRightsKeys[3];
	}

	void xorEnPassant(u64 &key, const Square square) noexcept
	{
		if (square <= SQUARE_NB)
			key ^= EnPassantKeys[fileOf(square)];
	}
}