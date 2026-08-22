#pragma once

#include <string>
#include <sstream>
#include <thread>

#include "Board.h"

/**
 * This is a wrapper for the actual Chess Engine in order to provide CLI support
 * It is also designed to be compatible with the UCI Protocol
 */
class Uci
{
	inline static std::thread _searchThread{};
	inline static usize _threadCount{ std::thread::hardware_concurrency() - 1u };
	inline static usize _hashSizeMb{ 64 };
	inline static Board _board{};

public:
	static void init();
	static void loop();

private:
	static void printEngineInfo();
	static void setOption(std::istringstream &is);
	static void parsePosition(std::istringstream &is);
	static void parseGo(std::istringstream &is);
};
