#include "Evaluation.h"

#include <array>
#include <sstream>
#include <iomanip>

#include "../Stats.h"
#include "../Psqt.h"
#include "../PawnStructureTable.h"

namespace
{
	/*
	 * Most of these values were imported from the Stockfish Chess Engine
	 */
	constexpr Score PAWN_DOUBLED{ 10, 55 };
	constexpr Score PAWN_ISOLATED{ 3, 15 };
	constexpr short PAWN_CONNECTED[]{ 0, 5, 7, 11, 24, 48, 86 };
	constexpr Score PASSED_PAWN_RANK[] = {
		{}, { 7, 27 }, { 16, 32 }, { 17, 40 }, { 64, 71 }, { 170, 174 }, { 278, 262 }
	};

	constexpr Score MINOR_PAWN_SHIELD{ 18, 3 };
	constexpr Score THREATS_BY_MINOR[] = {
		{}, { 5, 32 }, { 55, 41 }, { 77, 56 }, { 89, 119 }, { 79, 162 }, {}
	};
	constexpr Score KING_PROTECTOR[] = {{ 8, 9 }, { 6, 9 }};
	constexpr Score HANGING{ 69, 36 };
	constexpr Score RESTRICTED_PIECE_MOVEMENT{ 7, 7 };
	constexpr Score WEAK_QUEEN_PROTECTION{ 14, 0 };
	constexpr Score THREAT_BY_KING{ 24, 89 };
	constexpr Score THREAT_BY_SAFE_PAWN{ 173, 94 };
	constexpr Score QUEEN_THREAT_BY_KNIGHT{ 16, 11 };
	constexpr Score QUEEN_THREAT_BY_SLIDER{ 60, 18 };
	constexpr Score BISHOP_XRAY_PAWNS{ 4, 5 };
	constexpr Score BISHOP_PAWNS[] = {
		{ 3, 8 }, { 3, 9 }, { 2, 8 }, { 3, 8 }
	};
	constexpr Score UNCONTESTED_OUTPOST{ 1, 10 };

	constexpr Score TRAPPED_ROOK{ 55, 13 };
	constexpr Score ROOK_ON_FILE[] = {
		{ 19, 7 }, { 48, 27 }
	};
	constexpr Score THREAT_BY_ROOK[] = {
		{}, { 3, 44 }, { 37, 68 }, { 42, 60 }, { 0, 39 }, { 58, 43 }, {}
	};

	constexpr Score KING_RING_THREAT_BISHOP{ 24, 0 };
	constexpr Score KING_RING_THREAT_ROOK{ 16, 0 };

	// King Safety
	constexpr Score KING_PAWN_SHIELD{ 26, 4 };

	// Mobility
	constexpr Score KNIGHT_MOBILITY[] = {
		{ -62, -79 }, { -53, -57 }, { -12, -31 }, { -3, -17 }, { 3, 7 },
		{ 12, 13 }, { 21, 16 }, { 28, 21 }, { 37, 26 }
	};
	constexpr Score BISHOP_MOBILITY[] = {
		{ -47, -59 }, { -20, -25 }, { 14, -8 }, { 29, 12 }, { 39, 21 },
		{ 53, 40 }, { 53, 56 }, { 60, 58 }, { 62, 65 }, { 69, 72 }, { 78, 78 },
		{ 83, 87 }, { 91, 88 }, { 96, 98 }
	};
	constexpr Score ROOK_MOBILITY[] = {
		{ -60, -82 }, { -24, -15 }, { 0, 17 }, { 3, 43 }, { 4, 72 },
		{ 14, 100 }, { 20, 102 }, { 30, 122 }, { 41, 133 }, { 41, 139 },
		{ 41, 153 }, { 45, 160 }, { 57, 165 }, { 58, 170 }, { 67, 175 }
	};
	constexpr Score QUEEN_MOBILITY[] = {
		{ -29, -49 }, { -16, -29 }, { -8, -8 }, { -8, 17 }, { 18, 39 }, { 25, 54 },
		{ 23, 59 }, { 37, 73 }, { 41, 76 }, { 54, 95 }, { 65, 95 }, { 68, 101 },
		{ 69, 124 }, { 70, 128 }, { 70, 132 }, { 70, 133 }, { 71, 136 }, { 72, 140 },
		{ 74, 147 }, { 76, 149 }, { 90, 153 }, { 104, 169 }, { 105, 171 }, { 106, 171 },
		{ 112, 178 }, { 114, 185 }, { 114, 187 }, { 119, 221 }
	};
}

static thread_local PawnStructureTable PawnTable{ 2 };

constexpr auto MASK_PAWN_SHIELD = []
{
	std::array<std::array<Bitboard, SQUARE_NB>, COLOR_NB> array{};

	for (u8 square{}; square < SQUARE_NB; ++square)
	{
		const auto bb = Bitboard::fromSquare(toSquare(square));

		array[WHITE][square] = Bitboard{
			(bb << 8 | (bb << 7 & ~FILE_H) | (bb << 9 & ~FILE_A)) & RANK_2 };
		array[BLACK][square] = Bitboard{
			(bb >> 8 | (bb >> 7 & ~FILE_A) | (bb >> 9 & ~FILE_H)) & RANK_7 };
	}

	return array;
}();

template <bool Trace>
struct Eval
{
	struct Traces
	{
		// Pieces
		std::array<Score, COLOR_NB> pawns{};
		std::array<Score, COLOR_NB> knights{};
		std::array<Score, COLOR_NB> bishops{};
		std::array<Score, COLOR_NB> rooks{};
		std::array<Score, COLOR_NB> queen{};
		std::array<Score, COLOR_NB> king{};
		std::array<Score, COLOR_NB> mobility{};
		std::array<Score, COLOR_NB> piecesTotal{};

		std::array<Score, COLOR_NB> passedPawns{};
		std::array<Score, COLOR_NB> kingProtector{};
		std::array<Score, COLOR_NB> minorPawnShield{};

		// Threats
		std::array<Score, COLOR_NB> threatsByMinor{};
		std::array<Score, COLOR_NB> threatsByRook{};
		std::array<Score, COLOR_NB> threatsByKing{};
		std::array<Score, COLOR_NB> threatBySafePawn{};
		std::array<Score, COLOR_NB> piecesHanging{};
		std::array<Score, COLOR_NB> weakQueenProtection{};
		std::array<Score, COLOR_NB> queenThreatByKnight{};
		std::array<Score, COLOR_NB> queenThreatBySlider{};
		std::array<Score, COLOR_NB> restrictedMovement{};
		std::array<Score, COLOR_NB> attacksTotal{};

		std::array<Score, COLOR_NB> kingSafety{};

		std::array<Score, COLOR_NB> total{};
	};

	explicit Eval(const Board &board) noexcept
		: board(board)
	{
		if constexpr (Trace)
			_trace = new Traces{};

		const auto wKingSq = board.getKingSq(WHITE);
		const auto bKingSq = board.getKingSq(BLACK);
		const auto wKingRing = (Attacks::kingAttacks(wKingSq) | Bitboard::fromSquare(wKingSq))
							   & ~Attacks::pawnDoubleAttacks<WHITE>(board.getPieces(PAWN, WHITE));
		const auto bKingRing = (Attacks::kingAttacks(bKingSq) | Bitboard::fromSquare(bKingSq))
							   & ~Attacks::pawnDoubleAttacks<BLACK>(board.getPieces(PAWN, BLACK));
		_kingRing = { wKingRing, bKingRing };
	}

	~Eval() noexcept
	{
		if constexpr (Trace)
			delete _trace;
	}

	int computeValue() noexcept;

	[[nodiscard]] const auto &getTrace() const noexcept requires Trace
	{
		return *_trace;
	}

private:
	template <Color Us>
    [[nodiscard]] Score evaluatePieces() noexcept;
	template <Color Us>
	[[nodiscard]] Score evaluateAttacks() const noexcept;
	template <Color Us>
	[[nodiscard]] Score evaluatePawn(Square square) const noexcept;
	template <Color Us>
	[[nodiscard]] Score evaluateKnight(Square square) const noexcept;
	template <Color Us>
	[[nodiscard]] Score evaluateBishop(Square square) const noexcept;
	template <Color Us>
	[[nodiscard]] Score evaluateRook(Square square) const noexcept;
	template <Color Us>
	[[nodiscard]] Score evaluateQueen(Square square) const noexcept;
	template <Color Us>
	[[nodiscard]] Score evaluateKing() const noexcept;

	const Board &board;
	Traces *_trace = nullptr;
	std::array<std::array<Bitboard, 6>, COLOR_NB> _pieceAttacks{}; // No King
	std::array<Bitboard, COLOR_NB> _attacksMultiple{};
	std::array<Bitboard, COLOR_NB> _allAttacks{};
	std::array<Bitboard, COLOR_NB> _mobilityArea{};

	std::array<Bitboard, COLOR_NB> _kingRing{};
	// TODO
	std::array<i32, COLOR_NB> _kingAttacksCount{};
	std::array<i32, COLOR_NB> _kingAttackersCount{};
	std::array<i32, COLOR_NB> _kingAttackersWeight{};
};

int Evaluation::value(const Board &board) noexcept
{
	Stats::incBoardsEvaluated();
	return Eval<false>{ board }.computeValue();
}

int Evaluation::invertedValue(const Board &board) noexcept
{
	const int result = value(board);
	return board.colorToMove ? result : -result;
}

std::string Evaluation::traceValue(const Board &board)
{
	Eval<true> eval{ board };
	const int result = eval.computeValue();

	const auto &trace = eval.getTrace();
	std::ostringstream output;

	const auto traceElement = [&](const std::string_view name, const std::array<Score, COLOR_NB> &elem)
	{
		constexpr int MaxLetters = 24;
		constexpr int MaxDigits = 5;

		output << "| "
			   << std::setfill(' ') << std::setw(MaxLetters) << name << " | "
			   << std::setw(MaxDigits) << elem[WHITE].mg << ' '
			   << std::setw(MaxDigits) << elem[WHITE].eg << " | "
			   << std::setw(MaxDigits) << elem[BLACK].mg << ' '
			   << std::setw(MaxDigits) << elem[BLACK].eg << " | "
			   << std::setw(MaxDigits) << elem[WHITE].mg - elem[BLACK].mg << ' '
			   << std::setw(MaxDigits) << elem[WHITE].eg - elem[BLACK].eg << " |\n";
	};

	const auto Separator = "+--------------------------+-------------+-------------+-------------+\n";
	output << Separator
		   << "|                          |    WHITE    |    BLACK    |    TOTAL    |\n"
		   << "|                          |   MG   EG   |   MG   EG   |   MG   EG   |\n"
		   << Separator;

	traceElement("Pawns", trace.pawns);
	traceElement("Knights", trace.knights);
	traceElement("Bishops", trace.bishops);
	traceElement("Rooks", trace.rooks);
	traceElement("Queens", trace.queen);
	traceElement("King", trace.king);
	traceElement("Mobility", trace.mobility);
	traceElement("Pieces Total", trace.piecesTotal);
	traceElement("King Protectors", trace.kingProtector);
	traceElement("Minors Pawn Shield", trace.minorPawnShield);
	traceElement("Threats By Minor", trace.threatsByMinor);
	traceElement("Threats By Rook", trace.threatsByRook);
	traceElement("Threats By King", trace.threatsByKing);
	traceElement("Threats By Safe-Pawns", trace.threatBySafePawn);
	traceElement("Hanging Pieces", trace.piecesHanging);
	traceElement("Protection by Weak Queen", trace.weakQueenProtection);
	traceElement("Queen Threat By Knight", trace.queenThreatByKnight);
	traceElement("Queen Threat By Slider", trace.queenThreatBySlider);
	traceElement("Restricted Movement", trace.restrictedMovement);
	traceElement("Attacks Total", trace.attacksTotal);

	output << Separator;
	traceElement("Total", trace.total);
	output << "|                   Scaled |             |             | "
		   << std::setw(11) << result << " |\n"
		   << Separator;

	return output.str();
}

template <bool Trace>
int Eval<Trace>::computeValue() noexcept
{
	Score totalWhite = evaluatePieces<WHITE>();
	Score totalBlack = evaluatePieces<BLACK>();

	const auto kingScoreWhite = evaluateKing<WHITE>();
	const auto kingScoreBlack = evaluateKing<BLACK>();

	totalWhite += kingScoreWhite + evaluateAttacks<WHITE>();
	totalBlack += kingScoreBlack + evaluateAttacks<BLACK>();

	if constexpr (Trace)
	{
		_trace->king[WHITE] = kingScoreWhite;
		_trace->king[BLACK] = kingScoreBlack;
		_trace->total[WHITE] = totalWhite;
		_trace->total[BLACK] = totalBlack;
	}

	Score score = totalWhite - totalBlack;

	if (board.colorToMove)
		score += Evaluation::TEMPO_BONUS;
	else
		score -= Evaluation::TEMPO_BONUS;

	const Phase phase = board.getPhase();
	return (score.mg * phase + (score.eg * (128 - phase))) / 128;
}

template <bool Trace>
template <Color Us>
Score Eval<Trace>::evaluatePieces() noexcept
{
	constexpr Color Them = ~Us;
	Score score;

	_allAttacks[Us] = _kingRing[Us];

	// Bonuses for Knights and Bishops
	const auto knightBishopBonus = [&](const Square square)
	{
		constexpr Dir Behind = Us ? Dir::SOUTH : Dir::NORTH;
		const auto bb = Bitboard::fromSquare(square);

		const bool isBishop = (board.getPieces(BISHOP) & bb).notEmpty();

		const auto kingProtectorScore =
			KING_PROTECTOR[isBishop] * Bits::getDistanceBetween(square, board.getKingSq(Us));
		score -= kingProtectorScore;
		if constexpr (Trace)
			_trace->kingProtector[Us] -= kingProtectorScore;

		const auto pawns = board.getPieces(PAWN, Us).shift<Behind>();

		if ((bb & pawns).notEmpty())
		{
			score += MINOR_PAWN_SHIELD;

			if constexpr (Trace)
				_trace->minorPawnShield[Us] += MINOR_PAWN_SHIELD;
		}
	};

	const auto updateAttacks = [&](const PieceType type, const Bitboard attacks)
	{
		_pieceAttacks[Us][type] |= attacks;
		_attacksMultiple[Us] |= _allAttacks[Us] & attacks;
		_allAttacks[Us] |= attacks;
	};

	const auto updateKingAttacks = [&](const PieceType /*type*/, const Bitboard /*attacks*/)
	{
		/*constexpr Color Them = ~Us;
		if (attacks & _kingRing[Them])
		{
			++_kingAttackersCount[Us];
			_kingAttackersWeight[Us] += KING_ATTACK_WEIGHT[type];
			_kingAttacksCount[Us] += popCount(attacks & _kingRing[Them]);
		}*/
	};

	Score pawnScore;

	{
		Bitboard pawns = board.getPieces(PAWN, Us);

		if constexpr (Us == BLACK)
			pawns = pawns.flipVertical();

		const auto pawnsEntry = PawnTable[pawns.value()];

		// Don't use the Pawn Structure Table if we are Tracing the Eval
		if (Trace || pawnsEntry.pawns != pawns)
		{
			Bitboard bb = board.getPieces(PAWN, Us);
			while (bb.notEmpty())
				pawnScore += evaluatePawn<Us>(bb.popLsb());

			PawnTable.insert({ pawns, pawnScore });
		} else
			pawnScore = pawnsEntry.score;
	}

	// Pawns
	{
		const auto pawns = board.getPieces(PAWN, Us);
		const auto pawnsAttacks = Attacks::pawnAttacks<Us>(pawns);
		const auto doublePawnsAttacks = Attacks::pawnDoubleAttacks<Us>(pawns);

		_pieceAttacks[Us][PAWN] = pawnsAttacks;
		_attacksMultiple[Us] = doublePawnsAttacks | (doublePawnsAttacks & _allAttacks[Us]);
		_allAttacks[Us] |= pawnsAttacks;

		updateKingAttacks(PAWN, pawnsAttacks);
	}

	_mobilityArea[Us] =
		~board.getPieces(Us) & ~Attacks::pawnAttacks<Them>(board.getPieces(PAWN, Them));

	Bitboard pieces = board.getPieces(KNIGHT, Us);
	while (pieces.notEmpty())
	{
		const Square square = pieces.popLsb();
		const auto knightScore = evaluateKnight<Us>(square);
		knightBishopBonus(square);

		if constexpr (Trace)
			_trace->knights[Us] += knightScore;
		score += knightScore;

		const auto attacks = Attacks::knightAttacks(square);
		updateAttacks(KNIGHT, attacks);
		updateKingAttacks(KNIGHT, attacks);
	}

	pieces = board.getPieces(BISHOP, Us);
	while (pieces.notEmpty())
	{
		const Square square = pieces.popLsb();
		const auto bishopScore = evaluateBishop<Us>(square);
		knightBishopBonus(square);

		if constexpr (Trace)
			_trace->bishops[Us] += bishopScore;
		score += bishopScore;

		const auto attacks = Attacks::bishopAttacks(square, board.getPieces());
		updateAttacks(BISHOP, attacks);
		updateKingAttacks(BISHOP, attacks);
	}

	if (board.getPieceCount<BISHOP, Us>() >= 2)
		score += 40;

	pieces = board.getPieces(ROOK, Us);
	while (pieces.notEmpty())
	{
		const Square square = pieces.popLsb();
		const auto rookScore = evaluateRook<Us>(square);

		if constexpr (Trace)
			_trace->rooks[Us] += rookScore;
		score += rookScore;

		const auto attacks = Attacks::rookAttacks(square, board.getPieces());
		updateAttacks(ROOK, attacks);
		updateKingAttacks(ROOK, attacks);
	}

	pieces = board.getPieces(QUEEN, Us);
	while (pieces.notEmpty())
	{
		const Square square = pieces.popLsb();
		const auto queenScore = evaluateQueen<Us>(square);

		if constexpr (Trace)
			_trace->queen[Us] += queenScore;
		score += queenScore;

		const auto attacks = Attacks::queenAttacks(square, board.getPieces());
		updateAttacks(QUEEN, attacks);
		updateKingAttacks(QUEEN, attacks);
	}

	if constexpr (Trace)
	{
		_trace->pawns[Us] = pawnScore;
		_trace->piecesTotal[Us] = score + pawnScore;
	}

	return score + pawnScore;
}

/*
 * Needs to be called after both evaluatePieces<WHITE>() and evaluatePieces<BLACK>()
 */
template <bool Trace>
template <Color Us>
Score Eval<Trace>::evaluateAttacks() const noexcept
{
	constexpr Color Them = ~Us;
	Score totalValue{};

	const auto nonPawnEnemies = board.getPieces(Them) & ~board.getPieces(PAWN, Us);
	const auto stronglyProtected = _pieceAttacks[Them][PAWN]
								   | (_attacksMultiple[Them] & ~_attacksMultiple[Us]);
	const auto poorlyDefended = board.getPieces(Them) & ~stronglyProtected & _allAttacks[Us];
	const auto defended = nonPawnEnemies & stronglyProtected;

	if ((poorlyDefended | defended).notEmpty())
	{
		Bitboard minorThreats =
			(defended | poorlyDefended) &
			(_pieceAttacks[Us][KNIGHT] | _pieceAttacks[Us][BISHOP]) &
			board.getPieces(Them);

		while (minorThreats.notEmpty())
		{
			const auto value = THREATS_BY_MINOR[board.getSquare(minorThreats.popLsb()).type()];
			totalValue += value;
			if constexpr (Trace)
				_trace->threatsByMinor[Us] += value;
		}

		Bitboard rookThreats = poorlyDefended & _pieceAttacks[Us][ROOK] & board.getPieces(Them);
		while (rookThreats.notEmpty())
		{
			const auto value = THREAT_BY_ROOK[board.getSquare(rookThreats.popLsb()).type()];
			totalValue += value;
			if constexpr (Trace)
				_trace->threatsByRook[Us] += value;
		}

		if ((poorlyDefended & _kingRing[Us]).notEmpty())
		{
			totalValue += THREAT_BY_KING;
			if constexpr (Trace)
				_trace->threatsByKing[Us] += THREAT_BY_KING;
		}

		const auto hangingPieces = ~_allAttacks[Them] | (nonPawnEnemies & _attacksMultiple[Us]);
		const auto hangingScore = HANGING * i16((poorlyDefended & hangingPieces).count());

		totalValue += hangingScore;

		const auto weakQueenScore =
			WEAK_QUEEN_PROTECTION * (poorlyDefended & _pieceAttacks[Them][QUEEN]).count();
		totalValue += weakQueenScore;

		if constexpr (Trace)
		{
			_trace->piecesHanging[Us] = hangingScore;
			_trace->weakQueenProtection[Us] = weakQueenScore;
		}
	}

	// Bonus for threats on the next moves against enemy queen
	if (board.getPieceCount<QUEEN, Them>() == 1)
	{
		const bool imbalance =
			(board.getPieceCount<QUEEN, Them>() + board.getPieceCount<QUEEN, Us>()) == 1;

		const auto sq = board.getPieces(QUEEN, Them).bitScanForward();
		const auto safeSpots =
			_mobilityArea[Us] & ~stronglyProtected & ~board.getPieces(PAWN, Them);

		const auto knightAttacks = _pieceAttacks[Us][KNIGHT] & Attacks::knightAttacks(sq);
		const auto sliderAttacks =
			(_pieceAttacks[Us][BISHOP] & Attacks::bishopAttacks(sq, board.getPieces()))
			| (_pieceAttacks[Us][ROOK] & Attacks::rookAttacks(sq, board.getPieces()));

		const auto knightAttacksScore =
			QUEEN_THREAT_BY_KNIGHT * (knightAttacks & safeSpots).count();
		const auto sliderAttacksScore =
			QUEEN_THREAT_BY_SLIDER *
			(sliderAttacks & safeSpots & _attacksMultiple[Us]).count() *
			(1 + imbalance);

		totalValue += knightAttacksScore + sliderAttacksScore;
		if constexpr (Trace)
		{
			_trace->queenThreatByKnight[Us] = knightAttacksScore;
			_trace->queenThreatBySlider[Us] = sliderAttacksScore;
		}
	}

	const auto safe = ~_allAttacks[Them] | _allAttacks[Us];
	const auto safePawnsAttacks =
		Attacks::pawnAttacks<Us>(board.getPieces(PAWN, Us) & safe) & nonPawnEnemies;
	const auto safePawnThreatScore = THREAT_BY_SAFE_PAWN * safePawnsAttacks.count();
	totalValue += safePawnThreatScore;

	const auto restrictedMovement = _allAttacks[Them] & _allAttacks[Us] & ~stronglyProtected;
	const Score restrictedMovementScore =
		RESTRICTED_PIECE_MOVEMENT * i16(restrictedMovement.count());
	totalValue += restrictedMovementScore;

	if constexpr (Trace)
	{
		_trace->threatBySafePawn[Us] = safePawnThreatScore;
		_trace->restrictedMovement[Us] = restrictedMovementScore;
		_trace->attacksTotal[Us] = totalValue;
	}

	return totalValue;
}

template <bool Trace>
template <Color Us>
Score Eval<Trace>::evaluatePawn(const Square square) const noexcept
{
	constexpr Color Them = ~Us;
	constexpr Dir ForwardDir = Us ? NORTH : SOUTH;
	constexpr Dir BehindDir = Us ? SOUTH : NORTH;

	Score value = PSQT[PAWN][square];

	const auto bb = Bitboard::fromSquare(square);
	const u8 rank = (Us ? rankOf(square) : 7u - rankOf(square)) - 1u;

	const auto adjacentFiles = Bitboard::fromAdjacentFiles(square);
	const auto opposed = board.getPieces(PAWN, Them) & Bitboard::fromDirection(ForwardDir, square);
	const auto neighbours = board.getPieces(PAWN, Us) & adjacentFiles;
	const auto connected = neighbours & Bitboard::fromRank(square);
	const auto support =
		neighbours & Bitboard::fromRank(bb.shift<BehindDir>().bitScanForward());
	const bool isDoubled = (bb.shift<BehindDir>() & board.getPieces(PAWN, Us)).notEmpty();

	if ((support | connected).notEmpty())
	{
		const int connectedScore = PAWN_CONNECTED[rank] * (2 + (connected).notEmpty() - (opposed).notEmpty())
								   + 21 * support.count();

		value += Score(connectedScore, connectedScore * (rank - 2) / 4);
	} else if (neighbours.empty())
		value -= PAWN_ISOLATED;

	if (support.empty() && isDoubled)
		value -= PAWN_DOUBLED;

	{ // Passed Pawn
		Bitboard rays = Bitboard::fromDirection(ForwardDir, square);
		rays |= rays.shift<WEST>() | rays.shift<EAST>();

		const bool isPassedPawn = (rays & board.getPieces(PAWN, Them)).empty();

		if (isPassedPawn)
		{
			value += PASSED_PAWN_RANK[rank];

			if constexpr (Trace)
				_trace->passedPawns[Us] += PASSED_PAWN_RANK[rank];
		}
	}

	return value;
}

template <bool Trace>
template <Color Us>
Score Eval<Trace>::evaluateKnight(const Square square) const noexcept
{
	constexpr Color Them = ~Us;
	constexpr Dir BehindDir = Us ? SOUTH : NORTH;
	constexpr Bitboard OutpostRanks{
		Us == WHITE ? RANK_4 | RANK_5 | RANK_6 : RANK_5 | RANK_4 | RANK_3 };

	const auto bb = Bitboard::fromSquare(square);
	const auto attacks = Attacks::knightAttacks(square);

	Score value = PSQT[KNIGHT][square];

	{
		const auto pawns = board.getPieces(PAWN);
		const auto bitboard =
			OutpostRanks & (_pieceAttacks[Us][PAWN] | pawns.shift<BehindDir>());
		const auto targets = board.getPieces(Them) & ~pawns;

		if ((bitboard & bb & ~CENTER_FILES).notEmpty() // on a side outpost
			&& (attacks & targets).empty()        // no relevant attacks
			&& (!(targets & ((bb & QUEEN_SIDE).notEmpty() ? QUEEN_SIDE : KING_SIDE)).several()))
			value += UNCONTESTED_OUTPOST *
					 (pawns & ((bb & QUEEN_SIDE).notEmpty() ? QUEEN_SIDE : KING_SIDE)).count();
	}

	const int mobility = (attacks & _mobilityArea[Us]).count();
	value += KNIGHT_MOBILITY[mobility];

	if constexpr (Trace)
		_trace->mobility[Us] += mobility;

	return value;
}

template <bool Trace>
template <Color Us>
Score Eval<Trace>::evaluateBishop(const Square square) const noexcept
{
	constexpr Color Them = ~Us;
	constexpr Dir Down{ Us ? SOUTH : NORTH };

	const auto bb = Bitboard::fromSquare(square);
	Score value = PSQT[BISHOP][square];

	const i32 mobility = (Attacks::bishopAttacks(square, board.getPieces()) &
						  _mobilityArea[Us]).count();
	value += BISHOP_MOBILITY[mobility];

	if constexpr (Trace)
		_trace->mobility[Us] += mobility;

	// Penalty according to the number of our pawns on the same color square as the
	// bishop, bigger when the center files are blocked with pawns and smaller
	// when the bishop is outside the pawn chain.
	const auto blocked = board.getPieces(PAWN, Us) & board.getPieces().shift<Down>();

	const i32 pawnsOnTheSameColorSquares =
		(board.getPieces(PAWN, Us) &
		 ((DARK_SQUARES & bb).notEmpty() ? DARK_SQUARES : ~DARK_SQUARES)).count();
	value -=
		BISHOP_PAWNS[distanceToFileEdge(square)] *
		pawnsOnTheSameColorSquares
		* ((_pieceAttacks[Us][PAWN] & bb).empty() + (blocked & CENTER_FILES).count());

	// Enemy pawns x-rayed
	value -= BISHOP_XRAY_PAWNS *
			 (Attacks::bishopXRayAttacks(square) & board.getPieces(PAWN, Them)).count();

	// Long Diagonal Bishop
	const auto centerAttacks =
		Attacks::bishopAttacks(square, board.getPieces(PAWN, Them)) & CENTER_SQUARES;
	if (centerAttacks.several())
		value.mg += 45;

	// King Ring Threat
	if ((Attacks::bishopAttacks(square, board.getPieces(PAWN)) & _kingRing[Them]).notEmpty())
		value += KING_RING_THREAT_BISHOP;

	return value;
}

template <bool Trace>
template <Color Us>
Score Eval<Trace>::evaluateRook(const Square square) const noexcept
{
	constexpr Color Them = ~Us;

	Score value = PSQT[ROOK][square];

	const i32 mobility = (Attacks::rookAttacks(square, board.getPieces()) &
						  _mobilityArea[Us]).count();
	value += ROOK_MOBILITY[mobility];

	if constexpr (Trace)
		_trace->mobility[Us] += mobility;

	const auto file = Bitboard::fromFile(square);

	if (!(board.getPieces(PAWN, Us) & file).empty())
	{
		value += ROOK_ON_FILE[(board.getPieces(PAWN, Them) & file).empty()];
	} else if (mobility <= 3)
	{
		const u8 kingFile = fileOf(board.getKingSq(Us));

		if ((kingFile < 4) == (fileOf(square) < kingFile))
			value -= TRAPPED_ROOK * (1 + !board.canCastle<Us>());
	}

	if ((Bitboard::fromFile(square) & _kingRing[Them]).notEmpty())
		value += KING_RING_THREAT_ROOK;

	return value;
}

template <bool Trace>
template <Color Us>
Score Eval<Trace>::evaluateQueen(const Square square) const noexcept
{
	Score value = PSQT[QUEEN][square];

	const i32 mobility = (Attacks::queenAttacks(square, board.getPieces()) &
						  _mobilityArea[Us]).count();
	value += QUEEN_MOBILITY[mobility];

	if constexpr (Trace)
		_trace->mobility[Us] += mobility;

	constexpr Bitboard InitialPosition{ FILE_D & (Us ? RANK_1 : RANK_8) };
	if ((InitialPosition & Bitboard::fromSquare(square)).empty())
		value.mg -= 14;

	return value;
}

/*
 * Needs to be called after both evaluatePieces<WHITE>() and evaluatePieces<BLACK>()
 */
template <bool Trace>
template <Color Us>
Score Eval<Trace>::evaluateKing() const noexcept
{
	const u8 square = board.getKingSq(Us);

	Score value = PSQT[KNIGHT][square];

	if (board.isCastled<Us>())
		value.mg += 70;
	else
	{
		const short count = short(board.canCastleKs<Us>()) + short(board.canCastleQs<Us>());
		value.mg += count * 20;
	}

	value += KING_PAWN_SHIELD *
			 (MASK_PAWN_SHIELD[Us][square] & board.getPieces(PAWN, Us)).count();

	return value;
}
