#pragma once

#include <cstdint>

namespace bitfield_detail
{

	template <class T, std::size_t Size, std::size_t Offset>
	class field
	{
	public:
		static constexpr std::size_t Mask = (1u << Size) - 1u;

		inline static constexpr T get(const T &t)
		{
			auto offset = Offset;
			return (t >> offset) & Mask;
		}

		inline static constexpr void set(T &t, const T &value)
		{
			constexpr auto ShiftedMask = Mask << Offset;
			t = (t & ~ShiftedMask) | ((value & Mask) << Offset);
		}
	};

	template <std::size_t Index, std::size_t Size0>
	consteval std::size_t field_size() { return Size0; }

	template <std::size_t Index, std::size_t Size0, std::size_t Size1, std::size_t... Sizes>
	consteval std::size_t field_size()
	{
		return Index ? field_size<Index - bool(Index), Size1, Sizes...>() : Size0;
	}

	template <std::size_t Index, std::size_t MaxIndex, std::size_t Size0>
	consteval std::size_t field_offset()
	{
		if constexpr (Index < MaxIndex)
			return Size0;
		return {};
	}

	template <std::size_t Index, std::size_t MaxIndex, std::size_t Size0, std::size_t Size1, std::size_t... Sizes>
	consteval std::size_t field_offset()
	{
		if constexpr (Index < MaxIndex)
			return Size0 + field_offset<Index + 1, MaxIndex, Size1, Sizes...>();
		return {};
	}
}

template <class T, std::size_t... Sizes>
class Bitfield
{

	template <std::size_t Index>
	using bit_field_type = bitfield_detail::field<T,
		bitfield_detail::field_size<Index, Sizes...>(), bitfield_detail::field_offset<0, Index, Sizes...>()>;

public:
	constexpr Bitfield() = default;

	explicit constexpr Bitfield(const T t) : _t(t) {}

	template <std::size_t Index>
	constexpr T get() const { return bit_field_type<Index>::get(_t); }

	template <std::size_t Index, class A>
	constexpr A getAs() const { return static_cast<A>(get<Index>()); }

	template <std::size_t Index>
	constexpr void set(const T &value) { bit_field_type<Index>::set(_t, value); }

	template <std::size_t Index, class A>
	constexpr void setAs(const A &value) { set<Index>(static_cast<T>(value)); }

	[[nodiscard]] constexpr T value() const { return _t; }

private:
	T _t{};
};
