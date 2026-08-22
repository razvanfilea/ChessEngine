#pragma once

#include <array>
#include <optional>
#include <shared_mutex>

#include "Move.h"

class SearchEntry
{
	friend class TranspositionTable;

public:
	enum class Bound : u8
	{
		NONE [[maybe_unused]] = 0,
		EXACT = 0b01,
		ALPHA = 0b10,
		BETA = 0b11
	};

	SearchEntry() = default;

	constexpr SearchEntry(const u64 key, const int depth, const Move move, const bool qSearch, const Bound bound)
		: _key16(key >> 48u), _moveFromTo(move.getFromToBits()), _value(move.getScore()), _depth8(i8(depth))
	{
		_field.setAs<1>(bound);
		_field.setAs<2>(qSearch);
	}

	[[nodiscard]] constexpr u16 key() const noexcept { return _key16; }

	[[nodiscard]] constexpr Move move() const noexcept { return Move{ _moveFromTo, i32(_value) }; }

	[[nodiscard]] constexpr i32 depth() const noexcept { return i32(_depth8); }

	[[nodiscard]] constexpr u8 age() const noexcept { return _field.get<0>(); }

	[[nodiscard]] constexpr Bound bound() const noexcept { return _field.getAs<1, Bound>(); }

	[[nodiscard]] constexpr bool qSearch() const noexcept { return _field.getAs<2, bool>(); }

	constexpr void setAge(const u8 newAge) noexcept
	{
		_field.set<0>(newAge);
	}

private:
	u16 _key16{};
	u16 _moveFromTo{};
	i16 _value{};
	i8 _depth8{};

	/**
	 * 0 - age
	 * 1 - Bound
	 * 2 - qSearch
	 */
	Bitfield<u8, 5, 2, 1> _field{};

	static constexpr auto AGE_MASK = (1u << 5u) - 1u;
};

class TranspositionTable
{
	static constexpr usize CLUSTER_SIZE = 4;

	struct Cluster
	{
		std::array<SearchEntry, CLUSTER_SIZE> entries;
	};

	static_assert(sizeof(Cluster) == 32, "Wrong Cluster Size");

public:
	explicit TranspositionTable(usize sizeMb);

	TranspositionTable(const TranspositionTable &) = delete;
	TranspositionTable(TranspositionTable &&) = delete;
	~TranspositionTable();

	TranspositionTable &operator=(const TranspositionTable &) = delete;
	TranspositionTable &operator=(TranspositionTable &&) = delete;

	void prefetch(u64 zKey) const noexcept;
	[[nodiscard]] std::optional<SearchEntry> probe(u64 zKey) const noexcept;

	void insert(u64 zKey, SearchEntry entry) noexcept;
	bool setSize(usize sizeMb);
	void update() noexcept;
	[[nodiscard]] u8 currentAge() const noexcept;
	void clear() noexcept;

private:
	usize _size{};
	u64 _hashMask{};
	u8 _currentAge{};
	Cluster *_clusters = nullptr;
};
