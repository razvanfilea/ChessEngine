#include "Uci.h"

#include <iostream>

#include "Stats.h"
#include "Tests.h"
#include "algorithm/Search.h"
#include "MoveGen.h"
#include "polyglot/PolyBook.h"

void Uci::init()
{
	Attacks::init();
	_board.setToStartPos();
}

void Uci::loop()
{
	std::string line;
	std::string token;
	line.reserve(1024);
	token.reserve(64);

	printEngineInfo();

	_board.setToStartPos();

	while (std::getline(std::cin, line))
	{
		std::istringstream is(line);
		bool quit{};

		if (line.empty())
			continue;

		is >> token;

		// UCI Commands
		if (token == "isready")
			std::cout << "readyok\n";
		else if (token == "position" || token == "pos")
			parsePosition(is);
		else if (token == "go")
			parseGo(is);
		else if (token == "uci")
			printEngineInfo();
		if (token == "ucinewgame")
		{
			_board.setToStartPos();
			Search::clearAll();
			printEngineInfo();
		} else if (token == "setoption")
			setOption(is);
		else if (token == "board")
			std::cout << _board.toString() << std::endl;
		else if (token == "stop")
		{
			Search::stopSearch();
			if (_searchThread.joinable())
				_searchThread.join();
			std::cout << "Joined Thread\n";
		} else if (token == "quit")
			quit = true;

			// Non-UCI Commands
		else if (token == "usermove")
		{
			is >> token;
			if (token.empty())
				std::cout << "No move specified";
			else if (token == "undo")
				_board.undoMove();
			else if (const Move move = parseMove(_board, token); move.empty())
				std::cout << "Move could not be parsed";
			else if (!MoveList(_board).contains(move))
				std::cout << "Move is invalid";
			else if (!_board.isMoveLegal(move))
				std::cout << "Move is illegal";
			else
			{
				_board.makeMove(move);
				std::cout << "Move " << move.toString(true) << '\n';
			}

			std::cout << _board.toString() << std::endl;
		} else if (token == "polykey")
			std::cout << std::hex << "PolyKey: " << PolyBook::getKeyFromBoard(_board) << '\n' << std::dec;
		else if (token == "debug")
		{
			is >> token;

			if (token == "on")
			{
				Stats::setEnabled(true);
				std::cout << "Debug stats have been turned on";
			} else if (token == "off")
			{
				Stats::setEnabled(false);
				std::cout << "Debug stats have been turned off";
			} else
			{
				std::cout << (Stats::isEnabled() ? "Debug stats are on" : "Debug stats are off");
			}

			std::cout << std::endl;
		} else if (token == "evaltest")
		{
			const auto results = Tests::runEvaluationTests();
			if (results.empty())
				std::cout << "Test Completed Successfully\n";
			else
				std::cout << results;
		} else if (token == "perft")
		{
			i32 depth{};
			is >> depth;
			if (depth > 0)
				Tests::runPerftForPosition(_board.getFen(), depth);
			else
				Tests::runPerftTests();

		}

		std::cout.flush();

		if (quit)
			break;

		line.clear();
	}
}

void Uci::setOption(std::istringstream &is)
{
	std::string token;
	is >> token;

	if (token == "Threads")
	{
		usize threadCount{};
		is >> threadCount;
		_threadCount = std::clamp<usize>(threadCount, 1u, 128u);

		std::cout << "Thread Count has been set to " << _threadCount << std::endl;
	} else if (token == "Hash")
	{
		usize hashSize{};
		is >> hashSize;
		_hashSizeMb = std::clamp<usize>(hashSize, 2u, 8192u);

		std::cout << "Hash Size has been set to " << _hashSizeMb << "MB" << std::endl;
	} else if (token == "BookPath")
	{
		is >> token;
		if (token == "null")
			PolyBook::clearBook();
		else
			PolyBook::initBook(token);
	}
}

void Uci::parseGo(std::istringstream &is)
{
	i32 depth = MAX_DEPTH;
	i32 movesToGo = 35;
	i64 moveTime = -1;
	i64 time = -1;
	bool timeSet = false;
	i64 inc = 0;
	bool infinite = false;

	std::string token;

	while (is >> token)
	{
		if (_board.colorToMove == WHITE)
		{
			if (token == "wtime")
				is >> time;
			else if (token == "winc")
				is >> inc;
		} else
		{
			if (token == "btime")
				is >> time;
			else if (token == "binc")
				is >> inc;
		}

		if (token == "infinite")
			infinite = true;
		else if (token == "movestogo")
			is >> movesToGo;
		else if (token == "movetime")
			is >> moveTime;
		else if (token == "depth")
			is >> depth;
	}

	if (moveTime != -1)
	{
		timeSet = true;
		time = moveTime;
		movesToGo = 1;
	}

	if (time != -1)
	{
		timeSet = true;
		time /= movesToGo;
		time -= 50;
		time += inc / 2;
	}

	if (timeSet)
		time = std::max<i64>(5, time);

	if (infinite)
	{
		time = 0;
		timeSet = false;
	}

	std::cout << "time: " << time << " depth: " << depth
			  << " timeSet: " << std::boolalpha << timeSet << std::endl;

	const i64 searchTime = timeSet ? time : 0;
	const SearchOptions options{ depth, _threadCount, _hashSizeMb, true, searchTime };

	if (_searchThread.joinable())
		_searchThread.detach();
	Stats::resetStats();

	_searchThread = std::thread(Search::findBestMove, _board, options);
}

void Uci::parsePosition(std::istringstream &is)
{
	std::string token;
	is >> token;

	if (token == "fen")
	{
		std::string fen;
		fen.reserve(92);

		while (is >> token && !(token == "moves"))
		{
			fen += token;
			fen += ' ';
		}

		if (!fen.empty())
			_board.setToFen(fen);
	} else
	{
		// Load the default position anyhow
		_board.setToStartPos();
	}

	if (is >> token && token == "moves")
	{
		while (true)
		{
			if (!(is >> token))
				break;
			const Move move = parseMove(_board, token);
			if (!move.empty())
				_board.makeMove(move);
		}
	}

	std::cout << "Loaded Pos" << std::endl;
}

void Uci::printEngineInfo()
{
	std::cout << "id name LuckyEngine\n"
			  << "id author Filea (TheLuckyCoder) Filea Razvan\n\n"
              << "option name Threads type spin default 1 min 1 max 128\n"
              << "option name Hash type spin default 64 min 2 max 8192\n"
              << "option name BookPath type string\n"
			  << "uciok" << std::endl;
}
