#pragma once

#include <array>
#include <string>

#include "Piece.h"
#include "Move.h"
#include "algorithm/Attacks.h"

class BoardState final
{
public:
	u64 zKey{};

	Bitboard kingAttackers{};
	std::array<Bitboard, COLOR_NB> kingBlockers{};
	std::array<Bitboard, COLOR_NB> kingPinners{};
	std::array<Bitboard, PIECE_TYPE_NB> possibleCheckSquares{};

	u32 moveContents{};
	u8 castlingRights{};
	Square enPassantSq = SQ_NONE;
	u8 fiftyMoveRule{};

	[[nodiscard]] Move getMove() const noexcept { return Move{ moveContents }; }
};

class Board final
{
public:
	void setToStartPos();
	bool setToFen(const std::string &fen);
	[[nodiscard]] std::string getFen() const;

	// region Castling

	[[nodiscard]] bool canCastle(Color color) const noexcept;
	template <Color C>
	[[nodiscard]] bool canCastle() const noexcept;
	template <Color C>
	[[nodiscard]] bool canCastleKs() const noexcept;
	template <Color C>
	[[nodiscard]] bool canCastleQs() const noexcept;
	template <Color C>
	[[nodiscard]] bool isCastled() const noexcept;

	// endregion Castling

	// region Pieces
	[[nodiscard]] Piece getSquare(Square square) const noexcept;
	[[nodiscard]] Bitboard getPieces() const noexcept;
	[[nodiscard]] Bitboard getPieces(PieceType type, Color color) const noexcept;
    template <typename... Pieces>
	[[nodiscard]] Bitboard getPieces(PieceType type, Pieces... p) const noexcept;
	[[nodiscard]] Bitboard getPieces(Color color) const noexcept;
	template <PieceType P, Color C>
	[[nodiscard]] u8 getPieceCount() const noexcept;

	[[nodiscard]] Square getKingSq(Color color) const noexcept;

	// endregion Pieces

	// region State

	[[nodiscard]] u64 zKey() const noexcept;
	[[nodiscard]] Square getEnPassantSq() const noexcept;
	[[nodiscard]] CastlingRights getCastlingRights() const noexcept;

	[[nodiscard]] bool isDrawn() const noexcept;
	[[nodiscard]] Phase getPhase() const noexcept;

	// endregion State

	// region Move functions

	void makeMove(Move move) noexcept;
	void makeMove(Move move, bool moveGivesCheck) noexcept;
	void undoMove() noexcept;
	void makeNullMove() noexcept;
	void undoNullMove() noexcept;
	[[nodiscard]] bool doesMoveGiveCheck(Move move) const noexcept;
	[[nodiscard]] bool isMoveLegal(Move move) const noexcept;

	// endregion

	[[nodiscard]] Bitboard generateAttackers(Color attackerColor, Square sq, Bitboard blockers) const noexcept;
	[[nodiscard]] Bitboard generateAttackers(Square sq, Bitboard blockers) const noexcept;
	[[nodiscard]] Bitboard generateAttackers(Square sq) const noexcept;
	[[nodiscard]] bool isSideInCheck() const noexcept;

	void addPiece(Square square, Piece piece) noexcept;

private:
	void movePiece(Square from, Square to) noexcept;
	void removePiece(Square square) noexcept;
	Bitboard findBlockers(Bitboard sliders, Color color, Bitboard &pinners) const noexcept;

public:
	void computeCheckInfo() noexcept;
	[[nodiscard]] Bitboard getKingAttackers() const noexcept;
	[[nodiscard]] Bitboard getKingBlockers(Color color) const noexcept;

	[[nodiscard]] std::string toString() const noexcept;

private:
	std::array<Bitboard, COLOR_NB> piecesByColor{};
	std::array<Bitboard, PIECE_TYPE_NB> piecesByType{};
	std::array<u8, 15> pieceCount{};
	std::array<Piece, SQUARE_NB> squares{};

public:
	BoardState state{};
	i32 npm{};
	i16 ply{};
	Color colorToMove{};

private:
	i16 historyPly{};
	std::array<BoardState, MAX_MOVES> history{};
};

template <Color C>
bool Board::canCastle() const noexcept
{
	return state.castlingRights & (C == BLACK ? CASTLE_BLACK_BOTH : CASTLE_WHITE_BOTH);
}

template <Color C>
bool Board::canCastleKs() const noexcept
{
	return state.castlingRights & (C == BLACK ? CASTLE_BLACK_KING : CASTLE_WHITE_KING);
}

template <Color C>
bool Board::canCastleQs() const noexcept
{
	return state.castlingRights & (C == BLACK ? CASTLE_BLACK_QUEEN : CASTLE_WHITE_QUEEN);
}

template <Color C>
bool Board::isCastled() const noexcept
{
	return state.castlingRights & (C == BLACK ? CASTLED_BLACK : CASTLED_WHITE);
}

inline bool Board::canCastle(const Color color) const noexcept
{
	return color == BLACK ? canCastle<BLACK>() : canCastle<WHITE>();
}

force_inline Piece Board::getSquare(const Square square) const noexcept
{
	return squares[square];
}

force_inline Bitboard Board::getPieces() const noexcept
{
	return piecesByType[NO_PIECE_TYPE];
}

force_inline Bitboard Board::getPieces(const PieceType type, const Color color) const noexcept
{
	return piecesByColor[color] & piecesByType[type];
}

template <typename ... Pieces>
inline Bitboard Board::getPieces(const PieceType type, Pieces... p) const noexcept
{
    if constexpr (sizeof...(Pieces) > 0)
    {
        auto b = piecesByType[type];
        ((b |= piecesByType[p]), ...);
        return b;
    }
	return piecesByType[type];
}

force_inline Bitboard Board::getPieces(const Color color) const noexcept
{
	return piecesByColor[color];
}

template <PieceType P, Color C>
force_inline u8 Board::getPieceCount() const noexcept
{
	constexpr u8 p = static_cast<u8>(Piece{ P, C });
	return pieceCount[p];
}

force_inline Square Board::getKingSq(const Color color) const noexcept
{
	return getPieces(KING, color).bitScanForward();
}

force_inline u64 Board::zKey() const noexcept
{
	return state.zKey;
}

force_inline Square Board::getEnPassantSq() const noexcept
{
	return state.enPassantSq;
}

force_inline CastlingRights Board::getCastlingRights() const noexcept
{
	return CastlingRights(state.castlingRights);
}

force_inline Bitboard Board::getKingAttackers() const noexcept
{
	return state.kingAttackers;
}

force_inline Bitboard Board::getKingBlockers(const Color color) const noexcept
{
	return state.kingBlockers[color];
}
