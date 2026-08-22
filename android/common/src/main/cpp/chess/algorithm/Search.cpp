#include "Search.h"

#include <iostream>
#include <vector>

#include "../Stats.h"
#include "../Board.h"
#include "../MoveGen.h"
#include "../MoveOrdering.h"
#include "Evaluation.h"
#include "../Psqt.h"
#include "../polyglot/PolyBook.h"

static constexpr int WINDOW_MIN_DEPTH = 5;
static constexpr int WINDOW_SIZE = 12;

static constexpr int REVERSE_FUTILITY_MAX_DEPTH = 8;
static constexpr int REVERSE_FUTILITY_MARGIN = 220;

static constexpr int FUTILITY_QUIESCENCE_MARGIN = 100;
static constexpr int FUTILITY_MARGIN = 160;
static constexpr int FUTILITY_MAX_DEPTH = 8;

static thread_local Thread *localThreadInfo = nullptr;

auto &threadInfo() { return *localThreadInfo; }

SearchOptions Search::_searchOptions;
TranspositionTable Search::_transpositionTable{ _searchOptions.tableSizeMb() };
Search::SharedState Search::_sharedState{};

void Search::clearAll()
{
	_transpositionTable.clear();
	_sharedState.fullReset();
}

void Search::stopSearch()
{
	_sharedState.stopped = true;
}

bool Search::setTableSize(const usize sizeMb)
{
	return _transpositionTable.setSize(sizeMb);
}

Move Search::findBestMove(Board board, const SearchOptions &searchOptions)
{
	Stats::resetStats();
	// Apply SearchOptions
	_searchOptions = searchOptions;
	const auto threadCount = _searchOptions.threadCount();

	_transpositionTable.update();
	if (_searchOptions.tableSizeMb() != 0)
		setTableSize(_searchOptions.tableSizeMb());

	board.ply = 0;
	// Reset Depth Counter
	_sharedState.reset();

	Stats::restartTimer();

	if (PolyBook::isEnabled() && _sharedState.useBook)
	{
		const Move move = PolyBook::getBookMove(board);
		if (!move.empty())
		{
			std::cout << "bestmove " << move.toString() << std::endl;
			return move;
		} else
		{
			// No move has been found so no other moves will be found from here on out
			_sharedState.useBook = false; // This is reset in clearAll()
		}
	}

	const auto work = [&, board](const i32 threadId)
	{
		assert(threadId >= 1);
		assert(threadId <= i32(threadCount));
		localThreadInfo = new Thread(threadId, threadId == 1);

		while (!_sharedState.stopped)
		{
			const auto currentDepth = i32(_sharedState.depth);
			const auto depth = currentDepth + 1 + i32(Bits::bitScanForward(u64(threadId)));

			iterativeDeepening(board, std::min<i32>(depth, _searchOptions.depth()));
		}

		delete localThreadInfo;
		localThreadInfo = nullptr;
	};

	std::vector<std::thread> threads;
	threads.reserve(threadCount);

	for (usize i{}; i < threadCount; ++i)
		threads.emplace_back(work, i32(i) + 1);

	for (auto &&thread : threads)
		thread.join();

	const Move move = _sharedState.lastReportedBestMove;
	assert(!move.empty());
	return move;
}

void Search::printUci(Board &board)
{
	if (!threadInfo().mainThread)
		return;
	std::array<Move, MAX_DEPTH> pvArray{};

	const auto getPvLine = [&board, &pvArray](const int depth) -> int
	{
		assert(depth <= MAX_DEPTH);

		int count{};

		while (true)
		{
			const auto probedResult = _transpositionTable.probe(board.zKey());

			if (!probedResult.has_value() || count >= depth)
				break;

			const Move probedMove = probedResult->move();
			const MoveList moveList(board);

			for (const Move &fullMove : moveList)
			{
				if (fullMove.getFromToBits() == probedMove.getFromToBits())
				{
					if (board.isMoveLegal(fullMove))
					{
						board.makeMove(fullMove);
						pvArray[count++] = fullMove;
						break;
					}
				}
			}
		}

		while (board.ply > 0)
			board.undoMove();

		return count;
	};

	int depth;
	int bestScore;
	usize time;

	{
		std::lock_guard lock{ _sharedState.mutex };
		depth = _sharedState.depth;
		bestScore = _sharedState.bestScore;
		time = _sharedState.time;
	}

	const int lastReportedDepth = _sharedState.lastReportedDepth;
	if (!_sharedState.stopped && depth > lastReportedDepth)
	{
		const int pvMoves = getPvLine(depth);

		const int cp = bestScore * 100 / 213;
		std::cout << "info depth " << depth << " score cp " << cp
				  << " nodes " << _sharedState.nodes << " time " << time;

		std::cout << " pv: ";
		for (int pvCount = 0; pvCount < pvMoves; ++pvCount)
			std::cout << pvArray[pvCount].toString() << ' ';

		std::cout << std::endl;

		_sharedState.lastReportedDepth = depth;
		_sharedState.lastReportedBestMove = pvArray[0];

		if (depth >= _searchOptions.depth())
			stopSearch();
	}

	if (_sharedState.stopped)
	{
		if (_sharedState.lastReportedBestMove.empty())
		{
			MoveList moveList(board);
			MoveOrdering::sortQMoves(moveList);

			std::cout << "No move found, returning: " << moveList.front().toString() << '\n';
			_sharedState.lastReportedBestMove = moveList.front();
		}
		std::cout << "bestmove " << _sharedState.lastReportedBestMove.toString() << std::endl;
	}
}

void Search::iterativeDeepening(Board board, const int targetDepth)
{
	auto &&thread = threadInfo();
	int bestScore = VALUE_MIN;

	for (int currentDepth = 1; currentDepth <= targetDepth; ++currentDepth)
	{
		// Stop if we have found a mate value
		if (bestScore > VALUE_MATE_MAX_DEPTH /*&& !bestMove.empty()*/)
			break;

		thread.nodesCount = 0;

		bestScore = aspirationWindow(board, currentDepth, bestScore);
		_sharedState.nodes += thread.nodesCount;

		if (bestScore != VALUE_MIN && currentDepth > _sharedState.depth)
		{
			std::lock_guard lock{ _sharedState.mutex };
			if (currentDepth > _sharedState.depth)
			{
				_sharedState.depth = currentDepth;
				_sharedState.bestScore = bestScore;
				_sharedState.time = Stats::getElapsedMs();
			}
		}

		printUci(board);

		if (_sharedState.stopped)
			break;
	}
}

int Search::aspirationWindow(Board &board, const int depth, const int bestScore)
{
	int alpha = VALUE_MIN;
	int beta = VALUE_MAX;
	int delta = WINDOW_SIZE;

	if (depth >= WINDOW_MIN_DEPTH)
	{
		alpha = std::max<int>(VALUE_MIN, bestScore - delta);
		beta = std::min<int>(VALUE_MAX, bestScore + delta);
	}

	int adjustedDepth = depth;

	while (true)
	{
		const int searchedValue = search(board, alpha, beta, adjustedDepth, true, true, true);
		if (_sharedState.stopped)
			return 0;

		if (searchedValue <= alpha)
		{
			beta = (alpha + beta) / 2;
			alpha = std::max<int>(VALUE_MIN, alpha - delta);
			adjustedDepth = depth;
		} else if (searchedValue >= beta)
		{
			beta = std::min<int>(VALUE_MAX, beta + delta);
			adjustedDepth -= (abs(searchedValue) <= VALUE_MAX / 2);
		} else
		{
			// If the search returned a result within our window, return the value
			return searchedValue;
		}

		// Expand the search Window
		delta += delta / 2;
		assert(alpha >= VALUE_MIN && beta <= VALUE_MAX);
	}
}

int Search::search(Board &board, int alpha, int beta, const int depth, const bool isPvNode,
				   const bool doNull, const bool doLmr)
{
	const bool rootNode = !board.ply;
	if (rootNode)
		assert(isPvNode);

	// Try to prefetch the Transposition Table as soon as possible
	_transpositionTable.prefetch(board.zKey());

	if (checkTimeAndStop())
		return 0;

	++threadInfo().nodesCount;

	if (!rootNode)
	{
		if (board.isDrawn())
			return 0;

		if (board.ply >= MAX_DEPTH)
			return Evaluation::invertedValue(board);
	}

	if (depth <= 0)
		return _searchOptions.quietSearch()
			   ? searchCaptures(board, alpha, beta, depth)
			   : Evaluation::invertedValue(board);

	const int originalAlpha = alpha;
	const int startPly = board.ply;
	auto &&thread = threadInfo();

	const bool nodeInCheck = board.isSideInCheck();
	int eval = thread.evalStack[startPly] = VALUE_NONE;
	int improvement{};

	if (!nodeInCheck)
	{
		auto &&evalStack = thread.evalStack;
		evalStack[startPly] = eval = Evaluation::invertedValue(board);
		improvement =
			(startPly > 2 && evalStack[startPly - 2] != VALUE_NONE) ? (eval - evalStack[startPly - 2])
																	: (startPly > 4 &&
																	   evalStack[startPly - 4] != VALUE_NONE)
																	  ? (eval - evalStack[startPly - 4]) : 200;

		// Probe the Transposition Table
		const auto probeResult = _transpositionTable.probe(board.zKey());
		// Only cut with a greater depth and if this is not a PvNode
		if (probeResult.has_value()
			&& !probeResult->qSearch()
			&& probeResult->depth() >= depth
			&& (depth == 0 || !isPvNode))
		{
			const auto entryValue = probeResult->move().getScore();
			const auto entryBound = probeResult->bound();

			if (entryBound == SearchEntry::Bound::EXACT
				|| (entryBound == SearchEntry::Bound::BETA && entryValue >= beta)
				|| (entryBound == SearchEntry::Bound::ALPHA && entryValue <= alpha))
				return entryValue;
		}
	}

	// We are improving if our static eval increased in the last move
	const bool improving = startPly > 1 && improvement > 0;
	const int futilityMarginEval = eval + FUTILITY_MARGIN * depth;

	// Reverse Futility Pruning
	if (!isPvNode
		&& !nodeInCheck
		&& depth < REVERSE_FUTILITY_MAX_DEPTH
		&& (eval - REVERSE_FUTILITY_MARGIN * std::max(1, depth - improving)) >= beta
		&& eval < VALUE_MATE / 2)
		return eval;

	// Null Move Pruning
	if (!isPvNode
		&& doNull
		&& !nodeInCheck
		&& depth >= 4
		&& eval >= beta - 20 * depth - improvement / 15 + 204
		&& (board.getPieces(QUEEN, board.colorToMove) | board.getPieces(ROOK, board.colorToMove)).notEmpty())
	{
		board.makeNullMove();
		const int nullScore = -search(board, -beta, -beta + 1, depth - 4, false, false, false);
		board.undoNullMove();

		if (nullScore >= beta)
		{
			Stats::incNullCuts();
			return beta;
		}
	}

	MoveList moveList(board);
	MoveOrdering::sortMoves(thread, board, moveList);

	usize legalCount{};
	usize searchedCount{};
	int bestScore = Value::VALUE_MIN;
	Move bestMove;

	while (!moveList.empty())
	{
		assert(startPly == board.ply);

		const Move move = MoveOrdering::getNextMove(moveList);
		const bool pvMove = move.flags().pvMove();

		if (!board.isMoveLegal(move))
			continue;
		++legalCount;

		const bool moveGivesCheck = board.doesMoveGiveCheck(move);

		// Futility Pruning
		if (depth < FUTILITY_MAX_DEPTH
			&& !pvMove
			&& searchedCount > 3
			&& futilityMarginEval <= alpha
			&& !isMateValue(static_cast<Value>(alpha))
			&& !move.isTactical()
			&& !move.flags().doublePawnPush()
			&& !move.flags().promotion()
			&& !nodeInCheck
			&& !moveGivesCheck)
		{
			Stats::incFutilityCuts();
			if (futilityMarginEval > bestScore)
				bestScore = eval;
			continue;
		}

		board.makeMove(move, moveGivesCheck);

		int moveScore = alpha;
		bool doFullSearch = true;

		// Late Move Reductions
		if (!pvMove
			&& doNull
			&& doLmr
			&& searchedCount > 3
			&& depth >= 3
			&& board.ply > 4
			&& !move.isTactical()
			&& !nodeInCheck
			&& !moveGivesCheck)
		{
			moveScore = -search(board, -alpha - 1, -alpha, depth - 2, false, true, false);

			// Search with a full window if LMR failed high
			doFullSearch = moveScore > alpha;
			if (!doFullSearch)
				Stats::incLmrCount();
		}

		if (doFullSearch)
		{
			moveScore = -search(board, -beta, -alpha, depth - 1, pvMove, true, true);
		}

		++searchedCount;
		const int searchedPly = board.ply;
		board.undoMove();

		// Mate Pruning
		if (moveScore == Value::VALUE_MAX - searchedPly) // Winning
		{
			if (moveScore < beta)
			{
				beta = moveScore;
				if (alpha >= moveScore)
					return moveScore;
			}
		} else if (moveScore == Value::VALUE_MIN + searchedPly) // Losing
		{
			if (moveScore > alpha)
			{
				alpha = moveScore;
				if (beta <= moveScore)
					return moveScore;
			}
		}

		if (moveScore > bestScore)
		{
			bestScore = moveScore;
			bestMove = move;

			if (bestScore > alpha)
			{
				alpha = bestScore;
				thread.history[move.from()][move.to()] += depth;
			}
		}

		// Alpha-Beta Pruning
		if (bestScore >= beta)
		{
			if (!move.isTactical())
			{
				auto &searchKillers = thread.killers;
				searchKillers[1][startPly] = searchKillers[0][startPly];
				searchKillers[0][startPly] = move.getContents();
			}

			break;
		}
	}

	_transpositionTable.prefetch(board.zKey());

	if (legalCount == 0)
		return board.isSideInCheck() ? Value::VALUE_MIN + startPly : 0;

	Stats::incNodesSearched(searchedCount);

	// Store the results of search
	bestMove.setScore(bestScore);
	storeTTEntry(bestMove, board.zKey(), alpha, originalAlpha, beta, depth, false);

	assert(abs(bestScore) != VALUE_MIN);
	return alpha;
}

int Search::searchCaptures(Board &board, int alpha, int beta, const int depth)
{
	if (checkTimeAndStop())
		return 0;

	if (board.isDrawn())
		return 0;

	++threadInfo().nodesCount;

	const short startPly = board.ply;

	if (startPly >= MAX_DEPTH)
		return Evaluation::invertedValue(board);

	const bool nodeInCheck = board.isSideInCheck();

	const int originalAlpha = alpha;
	int standPat = Value::VALUE_MIN;
	int bestScore = Value::VALUE_MIN;
	Move bestMove;

	if (!nodeInCheck)
	{
		const auto probeResult = _transpositionTable.probe(board.zKey());
		if (probeResult.has_value()
			&& probeResult->depth() >= depth)
		{
			const i32 entryValue = probeResult->move().getScore();
			const auto bound = probeResult->bound();

			if (bound == SearchEntry::Bound::EXACT)
				return entryValue;
			if (bound == SearchEntry::Bound::ALPHA)
				alpha = std::max(alpha, entryValue);
			else if (bound == SearchEntry::Bound::BETA)
				beta = std::min(beta, entryValue);

			if (alpha >= beta)
				return entryValue;
		}

		standPat = Evaluation::invertedValue(board);

		alpha = std::max(alpha, standPat);
		if (alpha >= beta)
			return standPat;
	}

	MoveList moveList(board);

	MoveOrdering::sortQMoves(moveList);
	usize legalCount{};
	usize searchedCount{};

	const bool isEndGame = board.getPhase() < Phase::MIDDLE_GAME_PHASE / 3;

	while (!moveList.empty())
	{
		assert(startPly == board.ply);

		const Move move = MoveOrdering::getNextMove(moveList);

		if (!board.isMoveLegal(move))
			continue;
		++legalCount;

		if (!nodeInCheck // Look for all check evasions
			&& !move.isTactical()
			&& searchedCount)
		{
			break; // The moves are sorted so we can break if is not a capture
		}

		const int futilityEval = standPat + PSQT[move.piece()][move.to()].eg
								 + Evaluation::getPieceValue(move.promotedPiece())
								 + FUTILITY_QUIESCENCE_MARGIN;
		// Futility Pruning
		const auto moveGivesCheck = board.doesMoveGiveCheck(move);
		if (!nodeInCheck
			&& !isEndGame
			&& board.ply > 2
			&& standPat != VALUE_MIN
			&& searchedCount
			&& futilityEval <= alpha
			&& !moveGivesCheck)
		{
			Stats::incFutilityCuts();
			continue;
		}

		board.makeMove(move, moveGivesCheck);
		const int moveScore = -searchCaptures(board, -beta, -alpha, depth - 1);
		board.undoMove();
		++searchedCount;

		if (_sharedState.stopped)
			return 0;

		if (moveScore > bestScore)
		{
			bestScore = moveScore;
			bestMove = move;
		}

		if (bestScore >= beta)
			break;

		alpha = std::max(alpha, moveScore);
	}

	if (legalCount == 0)
		return board.isSideInCheck() ? Value::VALUE_MIN + board.ply : 0;

	if (!nodeInCheck)
	{
		bestMove.setScore(bestScore);
		storeTTEntry(bestMove, board.zKey(), alpha, originalAlpha, beta, depth, true);
	}

	Stats::incNodesSearched(searchedCount);

	return bestScore;
}

inline void Search::storeTTEntry(const Move &bestMove, const u64 key, const int alpha,
								 const int originalAlpha, const int beta, const int depth,
								 const bool qSearch)
{
	// Avoid putting an empty move in the Transposition Table if alpha was not raised
	assert(!bestMove.empty());

	// Store the result in the transposition table
	auto bound = SearchEntry::Bound::EXACT;
	if (alpha <= originalAlpha)
		bound = SearchEntry::Bound::ALPHA;
	else if (bestMove.getScore() >= beta)
		bound = SearchEntry::Bound::BETA;

	_transpositionTable.insert(key, SearchEntry{ key, i8(depth), bestMove, qSearch, bound });
}

bool Search::checkTimeAndStop()
{
	if (threadInfo().mainThread
		&& _searchOptions.isTimeSet()
		&& (threadInfo().nodesCount & 2047u) == 0
		&& Stats::getElapsedMs() >= _searchOptions.searchTime())
		stopSearch();

	return _sharedState.stopped;
}
