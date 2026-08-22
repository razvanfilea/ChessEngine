#include "TranspositionTable.h"

#include <cstring>
#include <iostream>

#ifdef _MSC_VER
#	include <xmmintrin.h>
#endif

static constexpr u64 MB = 1ull << 20;

TranspositionTable::TranspositionTable(const usize sizeMb)
{
	setSize(sizeMb);
}

TranspositionTable::~TranspositionTable()
{
	delete[] _clusters;
}

void TranspositionTable::prefetch(const u64 zKey) const noexcept
{
	assert(_clusters);

	const auto address = &_clusters[(zKey >> 48u) & _hashMask];
#ifdef _MSC_VER
	_mm_prefetch(reinterpret_cast<char*>(address), _MM_HINT_T0);
#else
	__builtin_prefetch(address);
#endif
}

std::optional<SearchEntry> TranspositionTable::probe(const u64 zKey) const noexcept
{
	assert(_clusters);

	const u16 key16 = zKey >> 48u;
	const auto index = zKey & _hashMask;

	assert(index < _size);
	auto &clusterEntries = _clusters[index].entries;

	for (usize i = 0; i < CLUSTER_SIZE; ++i)
	{
		auto &entry = clusterEntries[i];
		if (key16 == entry.key() && currentAge() == entry.age())
			return entry;
	}

	return {};
}

void TranspositionTable::insert(const u64 zKey, SearchEntry entry) noexcept
{
	assert(_clusters);

	const auto key = entry.key();
	const auto index = zKey & _hashMask;
	const auto tableAge = currentAge();

	assert(index < _size);
	auto &clusterEntries = _clusters[index].entries;
	auto *toReplace = &clusterEntries.front();

	usize i{};
	for (; i < CLUSTER_SIZE && key != clusterEntries[i].key(); ++i)
	{
		if (toReplace->depth() - (tableAge - toReplace->age())
			>= clusterEntries[i].depth() - (tableAge - clusterEntries[i].age()))
			toReplace = &clusterEntries[i];
	}

	toReplace = (i != CLUSTER_SIZE) ? &clusterEntries[i] : toReplace;

	// Don't overwrite an entry from the same position, unless we have
	// an exact bound or depth that is nearly as good as the old one
	if (key == toReplace->key()
		&& entry.bound() != SearchEntry::Bound::EXACT
		&& entry.depth() < toReplace->depth() - 3)
		return;

	entry.setAge(tableAge);
	*toReplace = entry;
}

bool TranspositionTable::setSize(usize sizeMb)
{
	const auto newSize = (sizeMb << 20u) / sizeof(Cluster);

	if (newSize == 0 || _size == newSize) return false;

	_size = newSize;
	delete[] _clusters;
	_clusters = nullptr;

	while (!_clusters && sizeMb)
	{
		const auto bytesSize = sizeof(Cluster) * _size;
		_clusters = static_cast<Cluster *>(operator new[](bytesSize, std::nothrow));

		if (_clusters)
			clear();
		else
		{
			std::cerr << "Failed to allocate " << sizeMb << "MB for the Transposition Table\n";
			sizeMb /= 2;
			_size = (sizeMb << 20u) / sizeof(Cluster);
		}
	}

	u64 keySize = 16u;
	while ((1ull << keySize) * sizeof(Cluster) <= sizeMb * MB / 2)
		++keySize;

	_hashMask = (1ull << keySize) - 1u;

	return true;
}

void TranspositionTable::update() noexcept
{
	if (_currentAge == SearchEntry::AGE_MASK)
	{
		clear();
	} else
	{
		(++_currentAge) &= SearchEntry::AGE_MASK;
	}
}

u8 TranspositionTable::currentAge() const noexcept
{
	return _currentAge;
}

void TranspositionTable::clear() noexcept
{
	std::memset(_clusters, 0, sizeof(Cluster) * _size);
	_currentAge = {};
}
