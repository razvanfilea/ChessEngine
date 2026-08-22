#pragma once

#include <sstream>

class Board;

class FenParser
{
public:
	static bool parseFen(Board &board, const std::string &fen);
	static std::string exportToFen(const Board &board);

private:
	static void parsePieces(Board &board, std::istringstream &stream);
};
