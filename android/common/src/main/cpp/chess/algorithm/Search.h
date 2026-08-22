#pragma once

#include "../SearchOptions.h"
#include "../Move.h"
#include "../TranspositionTable.h"

class Board;

class Search final
{
private:
	struct SharedState
	{
		bool stopped{};
		std::atomic_uint64_t nodes{};

		mutable std::mutex mutex{};
		// Stats for the last time the depth was updated
		std::atomic_int depth{};
		int bestScore{};
		i64 time{};

		// This should only be read and written by the main thread
		int lastReportedDepth{};
		Move lastReportedBestMove{};
		bool useBook{};

		void reset() noexcept
		{
			stopped = false;
			nodes = 0;
			depth = 0;
			bestScore = VALUE_MIN;
			time = 0;
			lastReportedDepth = 0;
		}

		void fullReset() noexcept
		{
			reset();
			useBook = true;
		}
	};

	static SearchOptions _searchOptions;
	static TranspositionTable _transpositionTable;
	static SharedState _sharedState;

public:
	Search() = delete;
	Search(const Search &) = delete;
	Search(Search &&) = delete;

	Search &operator=(const Search &) = delete;
	Search &operator=(Search &&) = delete;

	static void clearAll();
	static void stopSearch();
	static bool setTableSize(usize sizeMb);

	static Move findBestMove(Board board, const SearchOptions &searchOptions);

	static auto &getTranspTable() noexcept { return _transpositionTable; }

private:
	static void printUci(Board &board);
	static void iterativeDeepening(Board board, int targetDepth);
	static int aspirationWindow(Board &board, int depth, int bestScore);
	static int search(Board &board, int alpha, int beta, int depth, bool isPvNode,
					  bool doNull, bool doLmr);
	static int searchCaptures(Board &board, int alpha, int beta, int depth);

	inline static void storeTTEntry(const Move &bestMove, u64 key, int alpha, int originalAlpha,
									int beta, int depth, bool qSearch);
	static bool checkTimeAndStop();
};
