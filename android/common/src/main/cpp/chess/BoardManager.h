#pragma once

#include <atomic>
#include <functional>
#include <thread>
#include <mutex>
#include <vector>

#include "SearchOptions.h"
#include "Board.h"
#include "MoveGen.h"
#include "persistence/UndoRedoMoves.h"

enum class GameState : u8
{
	NONE,
	WINNER_WHITE,
	WINNER_BLACK,
	DRAW,
	WHITE_IN_CHECK,
	BLACK_IN_CHECK,
	INVALID = 10
};

/**
 * This is a wrapper for the actual Chess Engine in order to hopefully make it easier to be used with custom UIs
 */
class BoardManager final
{
public:
	using BoardChangedCallback = std::function<void(GameState state)>;
	using SearchFinishedCallback = std::function<void(bool success)>;

private:
	inline static std::recursive_mutex _mutex;
	inline static BoardChangedCallback _boardCallback;
	inline static SearchFinishedCallback _searchCallback;

	inline static std::atomic_bool _isBusy{ false };
	inline static constinit bool _isPlayerWhite{ true };
	inline static constinit Board _currentBoard{};
	inline static UndoRedo::MovesStack _movesStack;
	inline static SearchOptions _searchOptions;

public:
	static void initBoardManager(const BoardChangedCallback &callback, bool isPlayerWhite = true);
	static bool loadGame(bool isPlayerWhite, const std::string &fen);
	static bool loadGame(bool isPlayerWhite, const std::string &fen, const std::vector<Move> &moves);

	/// Actions
	static void makeMove(Move move);
	static void makeEngineMove();
	static void undoLastMoves();
	static void redoLastMoves();

	/// Getters and Setters
	static void setSearchFinishedCallback(const SearchFinishedCallback &callback)
	{
		std::lock_guard lock{ _mutex };
		_searchCallback = callback;
	}

	static bool isEngineBusy() noexcept { return _isBusy; }

	static bool isPlayerWhite() noexcept
	{
		std::lock_guard lock{ _mutex };
		return _isPlayerWhite;
	}

	static void setSearchOptions(const SearchOptions &searchOptions) noexcept
	{
		std::lock_guard lock{ _mutex };
		_searchOptions = searchOptions;
	}

	static SearchOptions getSearchOptions() noexcept
	{
		std::lock_guard lock{ _mutex };
		return _searchOptions;
	}

	static const auto &getBoard() noexcept
	{
		std::lock_guard lock{ _mutex };
		return _currentBoard;
	}

	static const UndoRedo::MovesStack &getMovesStack() noexcept
	{
		std::lock_guard lock{ _mutex };
		return _movesStack;
	}

	static std::vector<Move> getPossibleMoves(Square from);

private:
	static GameState getBoardState() noexcept;
};
