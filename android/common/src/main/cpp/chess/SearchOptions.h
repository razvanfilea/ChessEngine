#pragma once

#include <algorithm>
#include <thread>

#include "Defs.h"

class SearchOptions final
{
public:

	SearchOptions() : SearchOptions(6, std::max<usize>(1u, std::thread::hardware_concurrency() / 2u), 64, true, 10000)
	{
	}

	SearchOptions(const i32 depth,
				  const usize threadCount,
				  const usize tableSizeMb,
				  const bool quietSearch,
				  const i64 searchTime = {}) noexcept
		: _depth(std::clamp<i32>(depth, 2, MAX_DEPTH)),
		  _threadCount(std::clamp<usize>(threadCount, 1u, std::thread::hardware_concurrency())),
		  _cacheTableSizeMb(tableSizeMb),
		  _searchTime(searchTime),
		  _quiescenceSearch(quietSearch)
	{
	}

	[[nodiscard]] constexpr auto depth() const noexcept { return _depth; }

	[[nodiscard]] constexpr auto threadCount() const noexcept { return _threadCount; }

	[[nodiscard]] constexpr auto tableSizeMb() const noexcept { return _cacheTableSizeMb; }

	[[nodiscard]] constexpr bool isTimeSet() const noexcept { return _searchTime != 0ull; }

	[[nodiscard]] constexpr auto searchTime() const noexcept { return _searchTime; }

	[[nodiscard]] constexpr bool quietSearch() const noexcept { return _quiescenceSearch; }

private:
	i32 _depth;
	usize _threadCount;
	usize _cacheTableSizeMb;
	i64 _searchTime;
	bool _quiescenceSearch;
};
