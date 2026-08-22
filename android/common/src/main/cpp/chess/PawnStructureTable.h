#pragma once

#include "Bitboard.h"

struct PawnStructureEntry
{
	Bitboard pawns;
	Score score;
};

class PawnStructureTable
{
public:
	explicit PawnStructureTable(usize sizeMb);

	PawnStructureTable(const PawnStructureTable &) = delete;
	PawnStructureTable(PawnStructureTable &&) = delete;
	~PawnStructureTable() noexcept;

	PawnStructureTable &operator=(const PawnStructureTable &) = delete;
	PawnStructureTable &operator=(PawnStructureTable &&) = delete;

	PawnStructureEntry operator[](u64 key) const noexcept;

	void insert(const PawnStructureEntry &value) const noexcept;
	bool setSize(usize sizeMb);
	void clear() const noexcept;

private:
	usize _size{};
	PawnStructureEntry *_entries = nullptr;
};
