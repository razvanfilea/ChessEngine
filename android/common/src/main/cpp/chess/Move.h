#pragma once

#include <string>

#include "Defs.h"
#include "Bitfield.h"

class SimpleMove
{
public:
	class Flags final
	{
	public:
		enum Internal
		{
			CAPTURE = 1, // The move is a capture
			PROMOTION = 1 << 1, // The move is a promotion
			KSIDE_CASTLE = 1 << 2, // The move is a king side castle
			QSIDE_CASTLE = 1 << 3, // The move is a queen side castle
			DOUBLE_PAWN_PUSH = 1 << 4, // The move is a double pawn push
			EN_PASSANT = 1 << 5, // The move is an en passant capture (Do not set the CAPTURE flag too)
			PV_MOVE = 1 << 6, // The move is a PV Move
		};

	public:
		explicit constexpr Flags(const u8 flags) noexcept: _flags(flags) {}

		[[nodiscard]] constexpr u8 getContents() const noexcept { return _flags; }

		[[nodiscard]] constexpr bool capture() const noexcept { return _flags & Internal::CAPTURE; }

		[[nodiscard]] constexpr bool promotion() const noexcept { return _flags & Internal::PROMOTION; }

		[[nodiscard]] constexpr bool kSideCastle() const noexcept { return _flags & Internal::KSIDE_CASTLE; }

		[[nodiscard]] constexpr bool qSideCastle() const noexcept { return _flags & Internal::QSIDE_CASTLE; }

		[[nodiscard]] constexpr bool doublePawnPush() const noexcept { return _flags & Internal::DOUBLE_PAWN_PUSH; }

		[[nodiscard]] constexpr bool enPassant() const noexcept { return _flags & Internal::EN_PASSANT; }

		[[nodiscard]] constexpr bool pvMove() const noexcept { return _flags & Internal::PV_MOVE; }

	private:
		u8 _flags;
	};

	constexpr SimpleMove() = default;

	explicit constexpr SimpleMove(const u32 move) noexcept
		: _move(move)
	{
	}

	constexpr SimpleMove(const u8 from, const u8 to, const PieceType piece) noexcept
	{
		_move.set<0>(from);
		_move.set<1>(to);
		_move.setAs<2>(piece);
	}

	constexpr SimpleMove(const u8 from, const u8 to, const PieceType piece, const u8 flags) noexcept
		: SimpleMove(from, to, piece)
	{
		setFlags(flags);
	}

	[[nodiscard]] force_inline constexpr bool empty() const noexcept { return !static_cast<bool>(_move.value()); }

	[[nodiscard]] force_inline constexpr u32 getContents() const noexcept { return _move.value(); }

	[[nodiscard]] force_inline constexpr Square from() const noexcept
	{
		return toSquare(_move.get<0>());
	}

	[[nodiscard]] force_inline constexpr Square to() const noexcept
	{
		return toSquare(_move.get<1>());
	}

	[[nodiscard]] force_inline constexpr PieceType piece() const noexcept
	{
		return _move.getAs<2, PieceType>();
	}

	[[nodiscard]] force_inline constexpr PieceType capturedPiece() const noexcept
	{
		return _move.getAs<3, PieceType>();
	}

	force_inline constexpr void setCapturedPiece(const PieceType type) noexcept
	{
		_move.setAs<3>(type);
	}

	[[nodiscard]] force_inline constexpr PieceType promotedPiece() const noexcept
	{
		return _move.getAs<4, PieceType>();
	}

	force_inline constexpr void setPromotedPiece(const PieceType type) noexcept
	{
		_move.setAs<4>(type);
	}

	[[nodiscard]] force_inline constexpr Flags flags() const noexcept
	{
		return _move.getAs<5, Flags>();
	}

	force_inline constexpr void setFlags(const u8 flags) noexcept
	{
		_move.set<5>(flags);
	}

	[[nodiscard]] constexpr bool isTactical() const noexcept
	{
		const auto f = flags();
		return f.capture() || f.promotion();
	}

	[[nodiscard]] constexpr u16 getFromToBits() const noexcept
	{
		constexpr u16 Mask = (0x3Fu | (0x3Fu << 6u));
		return _move.value() & Mask;
	}

	constexpr bool operator==(const SimpleMove &rhs) const noexcept
	{
		return _move.value() == rhs._move.value();
	}

	constexpr bool operator!=(const SimpleMove &rhs) const noexcept
	{
		return _move.value() != rhs._move.value();
	}

	[[nodiscard]] std::string toString(const bool showPiece = false) const
	{
		std::string str;
		str.reserve(5);

		const Square fromSq = from();
		const Square toSq = to();

		if (showPiece)
		{
			const PieceType p = piece();
			char pChar = 'K';
			if (p == PAWN)
				pChar = 'P';
			else if (p == KNIGHT)
				pChar = 'N';
			else if (p == BISHOP)
				pChar = 'B';
			else if (p == ROOK)
				pChar = 'R';
			else if (p == QUEEN)
				pChar = 'Q';

			str += pChar;
		}

		str += static_cast<char>('a' + i8(fileOf(fromSq)));
		str += static_cast<char>('1' + i8(rankOf(fromSq)));

		str += static_cast<char>('a' + i8(fileOf(toSq)));
		str += static_cast<char>('1' + i8(rankOf(toSq)));

		if (flags().promotion())
		{
			const PieceType promoted = promotedPiece();
			if (showPiece)
				str.erase(0, 1);
			char p = 'Q';
			if (promoted == ROOK)
				p = 'R';
			else if (promoted == BISHOP)
				p = 'B';
			else if (promoted == KNIGHT)
				p = 'N';

			str += p;
		}

		return str;
	}

private:
	/*
	 * Bits 0 to 5 (6 bits) - 'from' square
	 * Bits 6 to 11 (6 bits) - 'to' square
	 * Bits 12 to 14 (3 bits) - 'moved' piece type
	 * Bits 15 to 17 (3 bits) - 'captured' piece type
	 * Bits 18 to 20 (3 bits) - 'promoted' piece type
	 * Bits 21 to 27 (7 bits) - 'flags'
	 */
	Bitfield<u32, 6, 6, 3, 3, 3, 7> _move{};
};

class Move : public SimpleMove
{
public:
	constexpr Move() = default;

	explicit constexpr Move(const SimpleMove &move) noexcept
		: SimpleMove(move)
	{
	}

	explicit constexpr Move(const u32 move, const i32 score = 0) noexcept
		: SimpleMove(move), _score(score)
	{
	}

	constexpr Move(const u8 from, const u8 to, const PieceType piece, const u8 flags = 0u) noexcept
		: SimpleMove(from, to, piece, flags)
	{
	}

	[[nodiscard]] force_inline constexpr SimpleMove move() const noexcept { return SimpleMove(getContents()); }

	[[nodiscard]] force_inline constexpr i32 getScore() const noexcept { return _score; }

	force_inline constexpr void setScore(const int score) noexcept
	{
		_score = score;
	}

private:
	i32 _score{};
};
