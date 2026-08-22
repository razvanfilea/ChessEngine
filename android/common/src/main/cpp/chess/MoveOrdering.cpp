#include "MoveOrdering.h"

#include "algorithm/Evaluation.h"

namespace MoveOrdering
{
	static constexpr int NormalScore = 1000000;
	static constexpr int PvScore = NormalScore * 2;
	static constexpr int EnPassantScore = 105;

	static constexpr std::array VictimScore = { 0, 100, 200, 300, 400, 500, 600 };
	static constexpr auto MvaLvv = []
	{
		std::array<std::array<int, PIECE_TYPE_NB>, PIECE_TYPE_NB> array{};

		for (u8 attacker = PAWN; attacker <= KING; ++attacker)
		{
			for (u8 victim = PAWN; victim <= KING; ++victim)
			{
				array[victim][attacker] = VictimScore[victim] + 6 - VictimScore[attacker] / 100;
			}
		}

		return array;
	}();

	void sortMoves(const Thread &thread, const Board &board, MoveList &moveList) noexcept
	{
		const auto probeResult = Search::getTranspTable().probe(board.zKey());
		const Move pvMove = probeResult.has_value() ? probeResult->move() : Move{};

		for (Move &move : moveList)
		{
			const auto flags = move.flags();

			if (move.getFromToBits() == pvMove.getFromToBits())
			{
				move.setScore(PvScore);
				move.setFlags(move.flags().getContents() | Move::Flags::PV_MOVE);
			} else if (flags.capture())
				move.setScore(MvaLvv[move.capturedPiece()][move.piece()] + NormalScore);
			else if (flags.promotion())
				move.setScore(Evaluation::getPieceValue(move.promotedPiece()) + NormalScore);
			else if (flags.enPassant())
				move.setScore(EnPassantScore + NormalScore);
			else if (thread.killers[0][board.ply] == move.getContents())
				move.setScore(900000);
			else if (thread.killers[1][board.ply] == move.getContents())
				move.setScore(800000);
			else
				move.setScore(thread.history[move.from()][move.to()]);
		}
	}

	void sortQMoves(MoveList &moveList) noexcept
	{
		for (Move &move : moveList)
		{
			const auto flags = move.flags();

			if (flags.capture())
				move.setScore(MvaLvv[move.capturedPiece()][move.piece()]);
			else if (flags.promotion())
				move.setScore(Evaluation::getPieceValue(move.promotedPiece()));
			else if (flags.enPassant())
				move.setScore(EnPassantScore);
		}
	}

	Move getNextMove(MoveList &moveList) noexcept
	{
		Move *foundMove = moveList.begin();

		for (auto &&move : moveList)
		{
			if (move.getScore() > foundMove->getScore())
				foundMove = &move;
		}

		const Move bestMove = *foundMove;

		*foundMove = moveList.back();
		moveList.popBack();

		return bestMove;
	}
}
