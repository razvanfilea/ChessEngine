#include "PawnStructureTable.h"

#include <cstring>

PawnStructureTable::PawnStructureTable(const usize sizeMb)
{
	setSize(sizeMb);
}

PawnStructureTable::~PawnStructureTable() noexcept
{
	delete[] _entries;
}

PawnStructureEntry PawnStructureTable::operator[](const u64 key) const noexcept
{
	assert(_entries);
	return _entries[key % _size];
}

void PawnStructureTable::insert(const PawnStructureEntry &value) const noexcept
{
	assert(_entries);
	_entries[value.pawns.value() % _size] = value;
}

bool PawnStructureTable::setSize(const usize sizeMb)
{
	const auto newSize = (sizeMb << 20u) / sizeof(PawnStructureEntry);

	if (newSize == 0 || _size == newSize) return false;

	_size = newSize;
	delete[] _entries;
	_entries = new PawnStructureEntry[_size]();

	return true;
}

void PawnStructureTable::clear() const noexcept
{
	std::memset(_entries, 0, sizeof(PawnStructureEntry) * _size);
}
