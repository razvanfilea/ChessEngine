#pragma once

#include <optional>
#include <vector>

#include "../Board.h"

struct IndexedPiece
{
	IndexedPiece(const u8 id, const Square square, const Color pieceColor, const PieceType pieceType)
		: id(id), square(square), pieceColor(pieceColor), pieceType(pieceType)
	{
	}

	/**
	 * An unique ID assigned to this piece
	 */
	u8 id;
	/**
	 * The square on which the Piece is located
	 */
	Square square;
	Color pieceColor;
	PieceType pieceType;
};

using IndexedPieces = std::vector<IndexedPiece>;

namespace UndoRedo
{
	static std::vector<IndexedPiece> incrementIndexedPieces(
		IndexedPieces pieces, const Color colorToMove, const Move move) noexcept
	{
		const auto findPiece = [&](const Square square)
		{
			const auto it = std::find_if(pieces.begin(), pieces.end(), [&square](const IndexedPiece &piece)
			{
				return piece.square == square;
			});
			assert(it != pieces.end());
			return it;
		};

		const auto movePiece = [&](const Square from, const Square to) { findPiece(from)->square = to; };
		const auto promotePawn = [&](const Square square, const PieceType to) { findPiece(square)->pieceType = to; };
		const auto removePiece = [&](const Square square) { pieces.erase(findPiece(square)); };

		const Square to = move.to();
		const auto flags = move.flags();
		const Color side = ~colorToMove;

		// Handle en passant capture and castling
		if (flags.enPassant())
			removePiece(capturedEnPassantSq(side, to));
		else if (flags.kSideCastle())
			movePiece(shiftToKingRank(side, SQ_H1), shiftToKingRank(side, SQ_F1));
		else if (flags.qSideCastle())
			movePiece(shiftToKingRank(side, SQ_A1), shiftToKingRank(side, SQ_D1));

		if (const PieceType capturedType = move.capturedPiece();
			capturedType != PieceType::NO_PIECE_TYPE)
			removePiece(to);

		movePiece(move.from(), to);

		if (move.flags().promotion())
			promotePawn(to, move.promotedPiece());

		return pieces;
	}

	static IndexedPieces makeIndexedPieces(const Board &board) noexcept
	{
		IndexedPieces pieces;
		pieces.reserve(board.getPieces().count());

		for (Square square{}; square < SQUARE_NB; ++square)
		{
			const auto piece = board.getSquare(square);
			if (piece.isValid())
				pieces.emplace_back(square, square, piece.color(), piece.type());
		}

		return pieces;
	}

	class HistoryBoard
	{
	public:
		HistoryBoard(const Board &board, const Move &move)
			: _pieces(makeIndexedPieces(board)), _move(move), _colorToMove(board.colorToMove)
		{
		}

		HistoryBoard(const IndexedPieces &previousPieces, const Board &board, const Move &move)
			: _pieces(incrementIndexedPieces(previousPieces, board.colorToMove, move)), _move(move),
			  _colorToMove(board.colorToMove)
		{
		}

		const auto &getIndexedPieces() const noexcept { return _pieces; }

		Move getMove() const noexcept { return _move; }

		bool colorToMove() const noexcept { return _colorToMove; }

	private:
		IndexedPieces _pieces;
		Move _move;
		Color _colorToMove;
	};

	class MovesStack
	{
	public:
		MovesStack() = default;

		MovesStack(const Board &board)
			: _initialPieces(makeIndexedPieces(board)), _startFen(board.getFen())
		{
			_data.reserve(64);
		}

		void push(const Board &board, const Move move) noexcept
		{
			assert(!move.empty());

			if (_index < static_cast<i64>(_data.size()) && !_data.empty())
			{
				const auto it = _data.begin() + _index;
				_data.erase(it + 1, _data.end());
			}

			if (!_data.empty())
				_data.emplace_back(peek().getIndexedPieces(), board, move);
			else
				_data.emplace_back(_initialPieces, board, move);

			_index = _data.size() - 1u;
		}

		Move undo() noexcept
		{
			if (_index >= 0 && _index < static_cast<i64>(_data.size()))
			{
				const Move move = peek().getMove();
				--_index;
				return move;
			}

			return {};
		}

		Move redo() noexcept
		{
			if (_index + 1 < static_cast<i64>(_data.size()))
			{
				++_index;
				return peek().getMove();
			}

			return {};
		}

		auto size() const noexcept { return _data.size(); }

		const HistoryBoard &peek() const noexcept
		{
			if (_index < 0)
				return _data.front();
			if (_index >= static_cast<i64>(_data.size()))
				return _data.back();
			return _data[_index];
		}

		const auto &getIndexedPieces() const noexcept
		{
			return (_index >= 0 && _index < static_cast<i64>(_data.size()))
				   ? peek().getIndexedPieces() : _initialPieces;
		}

		const auto &getStartFen() const noexcept { return _startFen; }

		auto getCurrentIndex() const noexcept { return _index; }

		auto begin() const noexcept { return _data.begin(); }

		auto end() const noexcept { return _data.end(); }

	private:
		IndexedPieces _initialPieces;
		std::string _startFen;
		std::vector<HistoryBoard> _data;
		i64 _index{ -1l };
	};
}
