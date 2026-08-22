#include "Tests.h"

#include <array>
#include <chrono>
#include <iomanip>
#include <iostream>
#include <sstream>
#include <vector>
#include <string_view>

#include "Board.h"
#include "MoveGen.h"
#include "algorithm/Evaluation.h"

namespace Tests
{
	// region Evaluation

	static Board mirrorBoard(const Board &board)
	{
		static constexpr std::array<u8, SQUARE_NB> MirrorSquare = {
			56, 57, 58, 59, 60, 61, 62, 63,
			48, 49, 50, 51, 52, 53, 54, 55,
			40, 41, 42, 43, 44, 45, 46, 47,
			32, 33, 34, 35, 36, 37, 38, 39,
			24, 25, 26, 27, 28, 29, 30, 31,
			16, 17, 18, 19, 20, 21, 22, 23,
			8, 9, 10, 11, 12, 13, 14, 15,
			0, 1, 2, 3, 4, 5, 6, 7,
		};

		Board result;

		result.colorToMove = ~board.colorToMove;
		for (u8 sq{}; sq < SQUARE_NB; ++sq)
		{
			const auto piece = board.getSquare(toSquare(sq));
			if (piece)
				result.addPiece(toSquare(MirrorSquare[sq]), ~piece);
		}

		// Castling
		const u8 originalRights = board.getCastlingRights();
		u8 &rights = result.state.castlingRights;
		rights |= (originalRights & 0b111u) << 3u; // Black -> White
		rights |= originalRights >> 3u; // White -> Black

		if (board.getEnPassantSq() != SQ_NONE)
			result.state.enPassantSq = toSquare(MirrorSquare.at(u8(board.getEnPassantSq())));

		result.state.fiftyMoveRule = board.state.fiftyMoveRule;
		result.state.kingAttackers = result.generateAttackers(result.getKingSq(result.colorToMove))
									 & result.getPieces(~result.colorToMove);
		result.computeCheckInfo();

		return result;
	}

	std::string runEvaluationTests() noexcept
	{
		static constexpr std::array Positions = {
			"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq",
			"r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq",
			"r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq",
			"rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ",
			"r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w",
			"8/8/p1p5/1p5p/1P5p/8/PPP2K1p/4R1rk w",
			"1q1k4/2Rr4/8/2Q3K1/8/8/8/8 w",
			"7k/5K2/5P1p/3p4/6P1/3p4/8/8 w",
			"8/6B1/p5p1/Pp4kp/1P5r/5P1Q/4q1PK/8 w",
			"8/8/1p1r1k2/p1pPN1p1/P3KnP1/1P6/8/3R4 b",
			"1r2r1k1/1pqbbppp/p2p1n2/4p3/P3PP2/2N1BB2/1PP2QPP/R4R1K b",
			"r1bk1bnr/ppp2ppp/8/4n3/2P5/P3B3/1P3PPP/RN2KBNR w KQ",
			"8/7p/3k2p1/6P1/4KP2/8/7P/8 w",
			"r1b1r1k1/1p1n1pbp/2p1n1p1/q1P1p3/4P3/1PN1BNPP/2Q2PB1/1R1R2K1 w",
			"1rN1r1k1/1pq2pp1/2p1nn1p/p2p1B2/3P4/4P2P/PPQ1NPP1/2R2RK1 b",
			"3r1rk1/p1q2pbp/1np1p1p1/1p2P3/5P2/2N2Q1P/PPP3P1/3RRBK1 b",
			"r2r2k1/p3ppbp/1p4p1/3p4/3P4/2P1P2P/P3BPP1/2R2RK1 w",
			"2rq3r/pb1pbkpp/1p2pp2/n1P5/2P5/QP2BNPB/P3PP1P/3R1RK1 w",
			"1r1q1rk1/pp1bbppp/2n1p1n1/4P3/2BpN3/3P1N2/PP2QPPP/R1B1R1K1 b",
			"3r2k1/2p2ppp/1p1br3/pPn5/3PP3/P7/1B1N2PP/R3R1K1 w",
			"r4rk1/p2qn1bp/1pnp2p1/2p2p2/4PP1N/2PPB3/PP2QN1P/R4RK1 b",
			"r3r1k1/1b1n1p2/1q1p1n1p/2p1p2P/p1P3p1/P1QNPPB1/1P2B1P1/2KR3R w",
			"1rb1nrk1/2q1bppp/p1n1p3/2p1P3/2Pp1PP1/3P1NN1/P5BP/R1BQ1RK1 w",
			"r3r1k1/ppqbbpp1/2pp1nnp/3Pp3/2P1P3/5N1P/PPBN1PP1/R1BQR1K1 w",
			"5r1k/1q2rnpp/p4p2/1pp5/6Q1/1P3P2/PBP3PP/3RR1K1 w",
			"2r2k2/5p2/2Bp1b1r/2qPp1pp/PpN1P3/1P2Q3/5PPP/4R1K1 w",
			"1k1r4/pp1b1R2/3q2pp/4p3/2B5/4Q3/PPP2B2/2K5 b",
			"3r1k2/4npp1/1ppr3p/p6P/P2PPPP1/1NR5/5K2/2R5 w",
			"2q1rr1k/3bbnnp/p2p1pp1/2pPp3/PpP1P1P1/1P2BNNP/2BQ1PRK/7R b",
			"rnbqkb1r/p3pppp/1p6/2ppP3/3N4/2P5/PPP1QPPP/R1B1KB1R w KQkq",
			"r1b2rk1/2q1b1pp/p2ppn2/1p6/3QP3/1BN1B3/PPP3PP/R4RK1 w",
			"2r3k1/pppR1pp1/4p3/4P1P1/5P2/1P4K1/P1P5/8 w",
			"1nk1r1r1/pp2n1pp/4p3/q2pPp1N/b1pP1P2/B1P2R2/2P1B1PP/R2Q2K1 w",
			"4b3/p3kp2/6p1/3pP2p/2pP1P2/4K1P1/P3N2P/8 w",
			"2kr1bnr/pbpq4/2n1pp2/3p3p/3P1P1B/2N2N1Q/PPP3PP/2KR1B1R w",
			"3rr1k1/pp3pp1/1qn2np1/8/3p4/PP1R1P2/2P1NQPP/R1B3K1 b",
			"2r1nrk1/p2q1ppp/bp1p4/n1pPp3/P1P1P3/2PBB1N1/4QPPP/R4RK1 w",
			"r3r1k1/ppqb1ppp/8/4p1NQ/8/2P5/PP3PPP/R3R1K1 b",
			"r2q1rk1/4bppp/p2p4/2pP4/3pP3/3Q4/PP1B1PPP/R3R1K1 w",
			"rnb2r1k/pp2p2p/2pp2p1/q2P1p2/8/1Pb2NP1/PB2PPBP/R2Q1RK1 w",
			"2r3k1/1p2q1pp/2b1pr2/p1pp4/6Q1/1P1PP1R1/P1PN2PP/5RK1 w",
			"r1bqkb1r/4npp1/p1p4p/1p1pP1B1/8/1B6/PPPN1PPP/R2Q1RK1 w kq",
			"r2q1rk1/1ppnbppp/p2p1nb1/3Pp3/2P1P1P1/2N2N1P/PPB1QP2/R1B2RK1 b",
			"r1bq1rk1/pp2ppbp/2np2p1/2n5/P3PP2/N1P2N2/1PB3PP/R1B1QRK1 b",
			"3rr3/2pq2pk/p2p1pnp/8/2QBPP2/1P6/P5PP/4RRK1 b",
			"r4k2/pb2bp1r/1p1qp2p/3pNp2/3P1P2/2N3P1/PPP1Q2P/2KRR3 w",
			"3rn2k/ppb2rpp/2ppqp2/5N2/2P1P3/1P5Q/PB3PPP/3RR1K1 w",
			"2r2rk1/1bqnbpp1/1p1ppn1p/pP6/N1P1P3/P2B1N1P/1B2QPP1/R2R2K1 b",
			"r1bqk2r/pp2bppp/2p5/3pP3/P2Q1P2/2N1B3/1PP3PP/R4RK1 b kq",
			"r2qnrnk/p2b2b1/1p1p2pp/2pPpp2/1PP1P3/PRNBB3/3QNPPP/5RK1 w",
			"4Q3/6pk/2pq4/3p4/1p1P3p/1P1K1P2/1PP3P1/8 b",
			"8/5pk1/4p3/7Q/8/3q4/KP6/8 b",
			"r3bb2/P1q3k1/Q2p3p/2pPp1pP/2B1P3/2B5/6P1/R5K1 w",
			"r1b5/p2k1r1p/3P2pP/1ppR4/2P2p2/2P5/P1B4P/4R1K1 w",
			"6r1/1p3k2/pPp4R/K1P1p1p1/1P2Pp1p/5P1P/6P1/8 w",
			"1k2b3/4bpp1/p2pp1P1/1p3P2/2q1P3/4B3/PPPQN2r/1K1R4 w",
			"2kr3r/ppp1qpp1/2p5/2b2b2/2P1pPP1/1P2P1p1/PBQPB3/RN2K1R1 b",
			"6k1/2q3p1/1n2Pp1p/pBp2P2/Pp2P3/1P1Q1KP1/8/8 w",
			"5r2/pp1RRrk1/4Qq1p/1PP3p1/8/4B3/1b3P1P/6K1 w",
			"6k1/1q2rpp1/p6p/P7/1PB1n3/5Q2/6PP/5R1K w",
			"3r2k1/p6p/b2r2p1/2qPQp2/2P2P2/8/6BP/R4R1K w",
			"8/6Bp/6p1/2k1p3/4PPP1/1pb4P/8/2K5 b",
			"2r1rbk1/p1Bq1ppp/Ppn1b3/1Npp4/B7/3P2Q1/1PP2PPP/R4RK1 w",
			"r4rk1/ppq3pp/2p1Pn2/4p1Q1/8/2N5/PP4PP/2KR1R2 w",
			"6k1/p4pp1/Pp2r3/1QPq3p/8/6P1/2P2P1P/1R4K1 w",
			"8/2k5/2p5/2pb2K1/pp4P1/1P1R4/P7/8 b",
			"2r5/1r5k/1P3p2/PR2pP1p/4P2p/2p1BP2/1p2n3/4R2K b",
			"8/1R2P3/6k1/3B4/2P2P2/1p2r3/1Kb4p/8 w",
			"1q1r3k/3P1pp1/ppBR1n1p/4Q2P/P4P2/8/5PK1/8 w",
			"6k1/5pp1/pb1r3p/8/2q1P3/1p3N1P/1P3PP1/2R1Q1K1 b",
			"8/Bpk5/8/P2K4/8/8/8/8 w",
			"1r6/5k2/p4p1K/5R2/7P/8/6P1/8 w",
			"8/6k1/p4p2/P3q2p/7P/5Q2/5PK1/8 w",
			"8/8/6p1/3Pkp2/4P3/2K5/6P1/n7 w",
			"1k1r4/pp1b1R2/3q2pp/4p3/2B5/4Q3/PPP2B2/2K5 b",
			"q1rr1k/3bbnnp/p2p1pp1/2pPp3/PpP1P1P1/1P2BNNP/2BQ1PRK/7R b",
			"r1b2rk1/2q1b1pp/p2ppn2/1p6/3QP3/1BN1B3/PPP3PP/R4RK1 w",
			"2r3k1/pppR1pp1/4p3/4P1P1/5P2/1P4K1/P1P5/8 w",
			"1nk1r1r1/pp2n1pp/4p3/q2pPp1N/b1pP1P2/B1P2R2/2P1B1PP/R2Q2K1 w",
			"4b3/p3kp2/6p1/3pP2p/2pP1P2/4K1P1/P3N2P/8 w",
			"2kr1bnr/pbpq4/2n1pp2/3p3p/3P1P1B/2N2N1Q/PPP3PP/2KR1B1R w",
			"3rr1k1/pp3pp1/1qn2np1/8/3p4/PP1R1P2/2P1NQPP/R1B3K1 b",
			"2r1nrk1/p2q1ppp/bp1p4/n1pPp3/P1P1P3/2PBB1N1/4QPPP/R4RK1 w",
			"r3r1k1/ppqb1ppp/8/4p1NQ/8/2P5/PP3PPP/R3R1K1 b",
			"r2q1rk1/4bppp/p2p4/2pP4/3pP3/3Q4/PP1B1PPP/R3R1K1 w",
			"rnb2r1k/pp2p2p/2pp2p1/q2P1p2/8/1Pb2NP1/PB2PPBP/R2Q1RK1 w",
			"2r3k1/1p2q1pp/2b1pr2/p1pp4/6Q1/1P1PP1R1/P1PN2PP/5RK1 w",
			"r1bqkb1r/4npp1/p1p4p/1p1pP1B1/8/1B6/PPPN1PPP/R2Q1RK1 w kq",
			"r2q1rk1/1ppnbppp/p2p1nb1/3Pp3/2P1P1P1/2N2N1P/PPB1QP2/R1B2RK1 b",
			"r1bq1rk1/pp2ppbp/2np2p1/2n5/P3PP2/N1P2N2/1PB3PP/R1B1QRK1 b",
			"3rr3/2pq2pk/p2p1pnp/8/2QBPP2/1P6/P5PP/4RRK1 b",
			"r4k2/pb2bp1r/1p1qp2p/3pNp2/3P1P2/2N3P1/PPP1Q2P/2KRR3 w",
			"3rn2k/ppb2rpp/2ppqp2/5N2/2P1P3/1P5Q/PB3PPP/3RR1K1 w",
			"2r2rk1/1bqnbpp1/1p1ppn1p/pP6/N1P1P3/P2B1N1P/1B2QPP1/R2R2K1 b",
			"r1bqk2r/pp2bppp/2p5/3pP3/P2Q1P2/2N1B3/1PP3PP/R4RK1 b kq",
			"r2qnrnk/p2b2b1/1p1p2pp/2pPpp2/1PP1P3/PRNBB3/3QNPPP/5RK1 w"
		};

		std::ostringstream output;

		for (auto &&pos : Positions)
		{
			Board board;
			board.setToFen(pos);
			const Board mirroredBoard = mirrorBoard(board);

			const int boardEval = Evaluation::invertedValue(board);
			const int mirroredBoardEval = Evaluation::invertedValue(mirroredBoard);

			if (boardEval != mirroredBoardEval)
			{
				output << "Evaluation Asymmetrical for: " << pos << '\n'
					   << board.toString()
					   << mirroredBoard.toString() << '\n'
					   << Evaluation::traceValue(board)
					   << Evaluation::traceValue(mirroredBoard)
					   << '\n';
				break;
			}
		}

		return output.str();
	}

	// endregion Evaluation

	// region Perft

	struct PerftInfo
	{
		u64 nodes{};
		u64 captures{};
		u64 enPassant{};
		u64 castles{};
		u64 promotions{};
		u64 checks{};
		u64 doubleChecks{};

		constexpr PerftInfo &operator+=(const PerftInfo &rhs) noexcept
		{
			nodes += rhs.nodes;
			captures += rhs.captures;
			enPassant += rhs.enPassant;
			castles += rhs.castles;
			promotions += rhs.promotions;
			checks += rhs.checks;
			doubleChecks += rhs.doubleChecks;
			return *this;
		}
	};

	static void perft(Board &board, PerftInfo &info, const unsigned depth)
	{
		if (board.state.fiftyMoveRule > 99)
			return;

		if (depth == 0)
		{
			const auto flags = board.state.getMove().flags();
			++info.nodes;
			info.captures += flags.capture() || flags.enPassant();
			info.enPassant += flags.enPassant();
			info.castles += flags.kSideCastle() || flags.qSideCastle();
			info.promotions += flags.promotion();
			const auto kingAttackers = board.getKingAttackers();
			info.checks += kingAttackers.notEmpty();
			info.doubleChecks += kingAttackers.several();
			return;
		}

		MoveList moveList(board);
		moveList.keepLegalMoves();

		for (const Move move : moveList)
		{
			board.makeMove(move);
			perft(board, info, depth - 1);
			board.undoMove();
		}
	}

	static PerftInfo basePerft(std::ostringstream &out, Board &board, const unsigned depth)
	{
		PerftInfo info{};

		MoveList moveList(board);
		moveList.keepLegalMoves();

		for (const Move move : moveList)
		{
			PerftInfo localInfo{};
			board.makeMove(move);
			perft(board, localInfo, depth - 1);
			board.undoMove();

			info += localInfo;
			out << move.toString() << ": " << localInfo.nodes << '\n';
		}

		return info;
	}

	static void perftWrapper(std::string_view tag, std::string_view fen, const std::vector<u64> perftVector)
	{
		using std::setw;

		Board board;
		board.setToFen(std::string(fen));

		constexpr auto DepthW = 5;
		constexpr auto ColumnW = 12;
		constexpr std::string_view Pipe = " | ";

		const auto displayRow = [&](const u32 depth, const double time, const PerftInfo &info)
		{
			const auto expectedNodes = perftVector[depth];
			const auto wrongResult = expectedNodes != info.nodes;
			const auto nodes = std::to_string(info.nodes);

			std::cout << "| " << std::setfill(' ') << std::fixed << std::setprecision(1)
					  << setw(DepthW) << depth << Pipe
					  << setw(ColumnW) << time << Pipe
					  << setw(ColumnW) << expectedNodes << Pipe
					  << setw(ColumnW) << (wrongResult ? (std::string("!!!") + nodes) : nodes) << Pipe
					  << setw(ColumnW) << info.captures << Pipe
					  << setw(ColumnW) << info.enPassant << Pipe
					  << setw(ColumnW) << info.castles << Pipe
					  << setw(ColumnW) << info.promotions << Pipe
					  << setw(ColumnW) << info.checks << Pipe
					  << setw(ColumnW) << info.doubleChecks << Pipe;
		};

		std::cout << tag << ':' << '\n';
		std::cout << "| " << std::setfill(' ')
				  << setw(DepthW) << "Depth" << Pipe
				  << setw(ColumnW) << "Seconds" << Pipe
				  << setw(ColumnW) << "Expected" << Pipe
				  << setw(ColumnW) << "Nodes" << Pipe
				  << setw(ColumnW) << "Captures" << Pipe
				  << setw(ColumnW) << "E.P." << Pipe
				  << setw(ColumnW) << "Castles" << Pipe
				  << setw(ColumnW) << "Promotions" << Pipe
				  << setw(ColumnW) << "Checks" << Pipe
				  << setw(ColumnW) << "DoubleChecks" << Pipe << '\n';

		for (unsigned depth = 1; depth < perftVector.size(); ++depth)
		{
			std::ostringstream out;
			const auto startTime = std::chrono::high_resolution_clock::now();
			const PerftInfo info = basePerft(out, board, depth);

			const auto endTime = std::chrono::high_resolution_clock::now();
			const auto timeNeeded =
				std::chrono::duration<double, std::milli>(endTime - startTime).count() / 1000;

			displayRow(depth, timeNeeded, info);

			if (depth == perftVector.size() - 1 || info.nodes != perftVector[depth])
			{
				std::cout << '\n' << out.str();
//				break;
			}

			std::cout << std::endl;
		}

		std::cout.flush();
	}

	void runPerftTests() noexcept
	{
		perftWrapper("Position 1", "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq", {
			1, 20, 400, 8902, 197281, 4865609, 119060324
		});

		perftWrapper("Position 2", "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq", {
			1, 48, 2039, 97862, 4085603, 193690690
		});

		perftWrapper("Position 3", "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w", {
			1, 14, 191, 2812, 43238, 674624, 11030083, 178633661
		});

		/*perftWrapper("Position 4", "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/P2P2PP/r2Q1R1K w kq - 0 2", {
			1, 6, 264, 9467, 422333, 15833292, 706045033
		});*/

		perftWrapper("Position 4", "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1", {
			1, 6, 264, 9467, 422333, 15833292, 706045033
		});

		perftWrapper("Position 5", "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8", {
			1, 44, 1486, 62379, 2103487, 89941194
		});

		perftWrapper("Position 6", "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10", {
			1, 46, 2079, 89890, 3894594, 164075551
		});

		std::cout << "Perft tests execution finished" << std::endl;
	}

	void runPerftForPosition(const std::string &fen, const i32 depth)
	{
		perftWrapper("Perft", fen, std::vector<u64>(depth));
	}

	// endregion Perft
}
