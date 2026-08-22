#pragma once

#include <string>

#include "Defs.h"

namespace Tests
{
	std::string runEvaluationTests() noexcept;

	void runPerftTests() noexcept;

	void runPerftForPosition(const std::string &fen, i32 depth);
}
