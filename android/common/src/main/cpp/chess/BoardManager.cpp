#include "BoardManager.h"

#include <iostream>

#include "algorithm/Evaluation.h"
#include "MoveGen.h"
#include "algorithm/Search.h"

void BoardManager::initBoardManager(const BoardChangedCallback &callback, const bool isPlayerWhite)
{
	std::lock_guard lock{ _mutex };
	Attacks::init();
	Search::clearAll();

	_isPlayerWhite = isPlayerWhite;
	_currentBoard.setToStartPos();
	_movesStack = { _currentBoard };
	_boardCallback = callback;

	_boardCallback(GameState::NONE);
}

bool BoardManager::loadGame(const bool isPlayerWhite, const std::string &fen)
{
	std::lock_guard lock{ _mutex };
	Search::clearAll();

	if (_currentBoard.setToFen(fen))
	{
		_isPlayerWhite = isPlayerWhite;
		_movesStack = { _currentBoard };

		_boardCallback(getBoardState());

		return true;
	}

	return false;
}

bool BoardManager::loadGame(const bool isPlayerWhite, const std::string &fen, const std::vector<Move> &moves)
{
	assert(moves.size() < MAX_MOVES);

	std::lock_guard lock{ _mutex };
	if (!loadGame(isPlayerWhite, fen))
		return false;

	for (const Move &move : moves)
	{
		if (move.empty() || !moveExists(_currentBoard, move) || !_currentBoard.isMoveLegal(move))
			break;
		_currentBoard.makeMove(move);
		_movesStack.push(_currentBoard, move);
	}

	_boardCallback(getBoardState());

	return true;
}

void BoardManager::makeMove(const Move move)
{
	std::lock_guard lock{ _mutex };

	_currentBoard.makeMove(move);
	_movesStack.push(_currentBoard, move);
	_boardCallback(getBoardState());
}

void BoardManager::makeEngineMove()
{
	std::lock_guard lock{ _mutex };
	if (const auto state = getBoardState();
		!(state == GameState::NONE || state == GameState::WHITE_IN_CHECK || state == GameState::BLACK_IN_CHECK))
		return;

	if (_isBusy) return;
	_isBusy = true;

	const auto startZKey = _currentBoard.zKey();

	std::thread([startZKey]
				{
					std::unique_lock lock{ _mutex };
					const auto tempBoard = _currentBoard;
					const auto options = _searchOptions;

					// This will should start executing exactly after makeEngineMove()
					// if the key is not as the specified one, cancel the search

					if (startZKey != _currentBoard.zKey())
					{
						_searchCallback(false);
						return;
					}
					lock.unlock();

					const Move bestMove = Search::findBestMove(tempBoard, options);
					_isBusy = false;

					lock.lock();
					// Make sure the board has not changed in the time we were searching
					if (_searchCallback)
						_searchCallback(startZKey == _currentBoard.zKey());

					if (startZKey == _currentBoard.zKey())
						makeMove(bestMove);
					else
						std::cout << "Board was changed while searching, cannot make found move\n" << std::flush;
				}).detach();
}

void BoardManager::undoLastMoves()
{
	std::lock_guard lock{ _mutex };

	if (!_movesStack.undo().empty())
		_currentBoard.undoMove();
	if (_movesStack.peek().colorToMove() != isPlayerWhite() && !_movesStack.undo().empty())
		_currentBoard.undoMove();

	_boardCallback(getBoardState());
}

void BoardManager::redoLastMoves()
{
	std::lock_guard lock{ _mutex };

	if (const Move move = _movesStack.redo(); !move.empty())
		_currentBoard.makeMove(move);
	if (_movesStack.peek().colorToMove() != isPlayerWhite())
	{
		const Move move = _movesStack.redo();
		if (!move.empty())
			_currentBoard.makeMove(move);
		else
			makeEngineMove(); // If there was no move to redo, try to make one
	}

	_boardCallback(getBoardState());
}

std::vector<Move> BoardManager::getPossibleMoves(const Square from)
{
	std::lock_guard lock{ _mutex };

	Board tempBoard = _currentBoard;
	const MoveList allMoves(tempBoard);

	std::vector<Move> moves;
	moves.reserve(allMoves.size());

	for (const Move &move : allMoves)
		if (move.from() == from && tempBoard.isMoveLegal(move))
			moves.push_back(move);

	return moves;
}

GameState BoardManager::getBoardState() noexcept
{
	std::lock_guard lock{ _mutex };
	if (_currentBoard.isDrawn())
		return GameState::DRAW;

	const Color colorToMove = _currentBoard.colorToMove;

	const bool otherInCheck =
		(_currentBoard.generateAttackers(_currentBoard.getKingSq(~colorToMove)) &
		 _currentBoard.getPieces(colorToMove)).notEmpty();

	if (_currentBoard.isSideInCheck() && otherInCheck)
		return GameState::INVALID;

	MoveList moveList(_currentBoard);
	moveList.keepLegalMoves();

	if (moveList.empty())
	{
		const auto winner = (colorToMove == WHITE) ? GameState::WINNER_BLACK : GameState::WINNER_WHITE;
		return (_currentBoard.isSideInCheck() ? winner : GameState::DRAW);
	}

	if (_currentBoard.isSideInCheck())
		return (colorToMove == WHITE) ? GameState::WHITE_IN_CHECK : GameState::BLACK_IN_CHECK;

	return GameState::NONE;
}
