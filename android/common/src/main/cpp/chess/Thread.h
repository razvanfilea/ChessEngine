#pragma once

#include <array>

class Thread
{
public:
	using Killers = std::array<std::array<u32, MAX_DEPTH>, COLOR_NB>;
	using History = std::array<std::array<u16, SQUARE_NB>, SQUARE_NB>;
	using EvalStack = std::array<i32, MAX_DEPTH>;

	const usize threadId;
	const bool mainThread;
	Killers killers{};
	History history{};
	EvalStack evalStack{};

	usize nodesCount{};

	Thread(const std::size_t threadId, const bool mainThread)
		: threadId(threadId), mainThread(mainThread) {}

	void clear() noexcept
	{
		killers.fill({});
		history.fill({});
		evalStack.fill({});
	}
};
