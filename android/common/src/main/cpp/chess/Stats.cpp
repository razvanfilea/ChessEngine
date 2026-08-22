#include "Stats.h"

#include <sstream>

std::atomic_bool Stats::_statsEnabled{ false };
std::chrono::time_point<std::chrono::high_resolution_clock> Stats::_startTime;
std::atomic_size_t Stats::_boardsEvaluated;
std::atomic_size_t Stats::_nodesSearched;
std::atomic_size_t Stats::_nullCuts;
std::atomic_size_t Stats::_futilityCuts;
std::atomic_size_t Stats::_lmrCount;

void Stats::setEnabled(const bool enabled) noexcept
{
	_statsEnabled = enabled;
}

void Stats::resetStats() noexcept
{
	_boardsEvaluated = 0;
	_nodesSearched = 0;
	_nullCuts = 0;
	_futilityCuts = 0;
	_lmrCount = 0;
}

void Stats::incBoardsEvaluated() noexcept
{
	if (_statsEnabled)
		++_boardsEvaluated;
}

void Stats::incNodesSearched(const usize amount) noexcept
{
	if (_statsEnabled)
		_nodesSearched += amount;
}

void Stats::incNullCuts() noexcept
{
	if (_statsEnabled)
		++_nullCuts;
}

void Stats::incFutilityCuts() noexcept
{
	if (_statsEnabled)
		++_futilityCuts;
}

void Stats::incLmrCount() noexcept
{
	if (_statsEnabled)
		++_lmrCount;
}

void Stats::restartTimer() noexcept
{
	_startTime = std::chrono::high_resolution_clock::now();
}

i64 Stats::getElapsedMs() noexcept
{
	const auto currentTime = std::chrono::high_resolution_clock::now();
	return std::chrono::duration_cast<std::chrono::milliseconds>(currentTime - _startTime).count();
}

std::string Stats::formatStats(const char separator)
{
	std::stringstream stream;

	const auto timeMs = getElapsedMs();

	stream << "Elapsed Time: " << timeMs << "ms" << separator;

	if (_statsEnabled)
	{
		const auto boardsEvaluated = static_cast<usize>(_boardsEvaluated);
		const auto nodesSearched = static_cast<usize>(_nodesSearched);
		const auto nullCuts = static_cast<usize>(_nullCuts);
		const auto futilityCuts = static_cast<usize>(_futilityCuts);
		const auto lmrCount = static_cast<usize>(_lmrCount);
		const usize nps = timeMs ? static_cast<usize>(nodesSearched / (timeMs / 1000.0)) : 0ul;

		stream << "Boards Evaluated: " << boardsEvaluated << separator
			   << "Nodes Searched: " << nodesSearched << separator
			   << "Nps: " << nps << separator
			   << "Null: " << nullCuts << separator
			   << "Futility/LMR: " << futilityCuts << '/' << lmrCount << separator;
	}

	return stream.str();
}
