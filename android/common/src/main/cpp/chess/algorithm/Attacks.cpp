#include "Attacks.h"

static constexpr std::array<u8, SQUARE_NB> BishopIndexBits = {
	6, 5, 5, 5, 5, 5, 5, 6,
	5, 5, 5, 5, 5, 5, 5, 5,
	5, 5, 7, 7, 7, 7, 5, 5,
	5, 5, 7, 9, 9, 7, 5, 5,
	5, 5, 7, 9, 9, 7, 5, 5,
	5, 5, 7, 7, 7, 7, 5, 5,
	5, 5, 5, 5, 5, 5, 5, 5,
	6, 5, 5, 5, 5, 5, 5, 6
};

static constexpr std::array<u8, SQUARE_NB> RookIndexBits = {
	12, 11, 11, 11, 11, 11, 11, 12,
	11, 10, 10, 10, 10, 10, 10, 11,
	11, 10, 10, 10, 10, 10, 10, 11,
	11, 10, 10, 10, 10, 10, 10, 11,
	11, 10, 10, 10, 10, 10, 10, 11,
	11, 10, 10, 10, 10, 10, 10, 11,
	11, 10, 10, 10, 10, 10, 10, 11,
	12, 11, 11, 11, 11, 11, 11, 12
};

/**
 * Rook and bishop magic values for magic table lookups
 */
static constexpr std::array<u64, SQUARE_NB> BishopMagics = {
	0x89a1121896040240ull, 0x2004844802002010ull, 0x2068080051921000ull, 0x62880a0220200808ull, 0x4042004000000ull,
	0x100822020200011ull, 0xc00444222012000aull, 0x28808801216001ull, 0x400492088408100ull, 0x201c401040c0084ull,
	0x840800910a0010ull, 0x82080240060ull, 0x2000840504006000ull, 0x30010c4108405004ull, 0x1008005410080802ull,
	0x8144042209100900ull, 0x208081020014400ull, 0x4800201208ca00ull, 0xf18140408012008ull, 0x1004002802102001ull,
	0x841000820080811ull, 0x40200200a42008ull, 0x800054042000ull, 0x88010400410c9000ull, 0x520040470104290ull,
	0x1004040051500081ull, 0x2002081833080021ull, 0x400c00c010142ull, 0x941408200c002000ull, 0x658810000806011ull,
	0x188071040440a00ull, 0x4800404002011c00ull, 0x104442040404200ull, 0x511080202091021ull, 0x4022401120400ull,
	0x80c0040400080120ull, 0x8040010040820802ull, 0x480810700020090ull, 0x102008e00040242ull, 0x809005202050100ull,
	0x8002024220104080ull, 0x431008804142000ull, 0x19001802081400ull, 0x200014208040080ull, 0x3308082008200100ull,
	0x41010500040c020ull, 0x4012020c04210308ull, 0x208220a202004080ull, 0x111040120082000ull, 0x6803040141280a00ull,
	0x2101004202410000ull, 0x8200000041108022ull, 0x21082088000ull, 0x2410204010040ull, 0x40100400809000ull,
	0x822088220820214ull, 0x40808090012004ull, 0x910224040218c9ull, 0x402814422015008ull, 0x90014004842410ull,
	0x1000042304105ull, 0x10008830412a00ull, 0x2520081090008908ull, 0x40102000a0a60140ull,
};

static constexpr std::array<u64, SQUARE_NB> RookMagics = {
	0xa8002c000108020ull, 0x6c00049b0002001ull, 0x100200010090040ull, 0x2480041000800801ull, 0x280028004000800ull,
	0x900410008040022ull, 0x280020001001080ull, 0x2880002041000080ull, 0xa000800080400034ull, 0x4808020004000ull,
	0x2290802004801000ull, 0x411000d00100020ull, 0x402800800040080ull, 0xb000401004208ull, 0x2409000100040200ull,
	0x1002100004082ull, 0x22878001e24000ull, 0x1090810021004010ull, 0x801030040200012ull, 0x500808008001000ull,
	0xa08018014000880ull, 0x8000808004000200ull, 0x201008080010200ull, 0x801020000441091ull, 0x800080204005ull,
	0x1040200040100048ull, 0x120200402082ull, 0xd14880480100080ull, 0x12040280080080ull, 0x100040080020080ull,
	0x9020010080800200ull, 0x813241200148449ull, 0x491604001800080ull, 0x100401000402001ull, 0x4820010021001040ull,
	0x400402202000812ull, 0x209009005000802ull, 0x810800601800400ull, 0x4301083214000150ull, 0x204026458e001401ull,
	0x40204000808000ull, 0x8001008040010020ull, 0x8410820820420010ull, 0x1003001000090020ull, 0x804040008008080ull,
	0x12000810020004ull, 0x1000100200040208ull, 0x430000a044020001ull, 0x280009023410300ull, 0xe0100040002240ull,
	0x200100401700ull, 0x2244100408008080ull, 0x8000400801980ull, 0x2000810040200ull, 0x8010100228810400ull,
	0x2000009044210200ull, 0x4080008040102101ull, 0x40002080411d01ull, 0x2005524060000901ull, 0x502001008400422ull,
	0x489a000810200402ull, 0x1004400080a13ull, 0x4000011008020084ull, 0x26002114058042ull
};

/**
 * Rook and bishop sliding attack masks indexed by [square]
 * These do not include edge squares
 */
static constexpr auto BishopMasks = []
{
	constexpr Bitboard edgeSquares{ FILE_A | FILE_H | RANK_1 | RANK_8 };
	std::array<Bitboard, SQUARE_NB> masks{};

	for (Square square{}; square < SQUARE_NB; ++square)
	{
		masks[square] = (Bitboard::fromDirection(NORTH_EAST, square)
						 | Bitboard::fromDirection(NORTH_WEST, square)
						 | Bitboard::fromDirection(SOUTH_WEST, square)
						 | Bitboard::fromDirection(SOUTH_EAST, square)) & ~(edgeSquares);
	}

	return masks;
}();

static constexpr auto RookMasks = []
{
	std::array<Bitboard, SQUARE_NB> masks{};

	for (Square square{}; square < SQUARE_NB; ++square)
	{
		masks[square] = (Bitboard::fromDirection(NORTH, square) & ~RANK_8)
						| (Bitboard::fromDirection(SOUTH, square) & ~RANK_1)
						| (Bitboard::fromDirection(EAST, square) & ~FILE_H)
						| (Bitboard::fromDirection(WEST, square) & ~FILE_A);
	}

	return masks;
}();

constexpr u64 getBlockersFromIndex(const int index, Bitboard mask)
{
	u64 blockers{};
	const int bits = mask.count();

	for (int i = 0; i < bits; i++)
	{
		const u8 bitPos = mask.popLsb();
		if (index & (1 << i))
			blockers |= Bits::Squares[bitPos];
	}

	return blockers;
}

static constexpr auto KnightAttacks = []
{
	std::array<Bitboard, SQUARE_NB> moves{};

	for (u8 square{}; square < SQUARE_NB; ++square)
	{
		const auto bb = Bitboard::fromSquare(toSquare(square));
		moves[square] = bb.shift<WEST, WEST, SOUTH>()
						| bb.shift<WEST, WEST, NORTH>()
						| bb.shift<WEST, SOUTH, SOUTH>()
						| bb.shift<WEST, NORTH, NORTH>()
						| bb.shift<EAST, SOUTH, SOUTH>()
						| bb.shift<EAST, NORTH, NORTH>()
						| bb.shift<EAST, EAST, SOUTH>()
						| bb.shift<EAST, EAST, NORTH>();
	}

	return moves;
}();

static constexpr auto KingAttacks = []
{
	std::array<Bitboard, SQUARE_NB> moves{};

	for (u8 square{}; square < SQUARE_NB; ++square)
	{
		const auto bb = Bitboard::fromSquare(toSquare(square));

		// Vertical and Horizontal
		auto attacks = bb.shift<NORTH>() | bb.shift<SOUTH>() | bb.shift<EAST>() | bb.shift<WEST>();

		// Diagonals
		attacks |= bb.shift<NORTH_EAST>() | bb.shift<NORTH_WEST>() |
				   bb.shift<SOUTH_EAST>() | bb.shift<SOUTH_WEST>();

		moves[square] = attacks;
	}

	return moves;
}();

static std::array<std::array<Bitboard, 1024>, SQUARE_NB> BishopAttacks;

static std::array<std::array<Bitboard, 4096>, SQUARE_NB> RookAttacks;

static constexpr auto BishopXRayAttacks = []
{
	std::array<Bitboard, SQUARE_NB> moves{};

	for (Square square{}; square < SQUARE_NB; ++square)
	{
		moves[square] = Bitboard::fromDirection(NORTH_WEST, square)
						| Bitboard::fromDirection(NORTH_EAST, square)
						| Bitboard::fromDirection(SOUTH_WEST, square)
						| Bitboard::fromDirection(SOUTH_EAST, square);
	}

	return moves;
}();

static constexpr auto RookXRayAttacks = []
{
	std::array<Bitboard, SQUARE_NB> moves{};

	for (Square square{}; square < SQUARE_NB; ++square)
	{
		moves[square] = Bitboard::fromDirection(NORTH, square)
						| Bitboard::fromDirection(EAST, square)
						| Bitboard::fromDirection(SOUTH, square)
						| Bitboard::fromDirection(WEST, square);
	}

	return moves;
}();

void Attacks::init()
{
	static bool initialized = false;
	if (initialized) return;
	initialized = true;

	for (u8 square{}; square < SQUARE_NB; ++square)
	{
		const int indexBit = BishopIndexBits[square];

		// For all possible blockers for this square
		for (int blockerIndex{}; blockerIndex < (1 << indexBit); ++blockerIndex)
		{
			const u64 blockers = getBlockersFromIndex(blockerIndex, BishopMasks[square]);
			const u64 attacks = Bits::generateBishopAttacks(square, blockers);
			const u64 index = (blockers * BishopMagics[square]) >> (64 - indexBit);

			BishopAttacks[square][index] = Bitboard{ attacks };
		}
	}

	for (u8 square{}; square < SQUARE_NB; ++square)
	{
		const int indexBit = RookIndexBits[square];

		// For all possible blockers for this square
		for (int blockerIndex{}; blockerIndex < (1 << indexBit); ++blockerIndex)
		{
			const u64 blockers = getBlockersFromIndex(blockerIndex, RookMasks[square]);
			const u64 attacks = Bits::generateRookAttacks(square, blockers);
			const u64 index = (blockers * RookMagics[square]) >> (64 - indexBit);

			RookAttacks[square][index] = Bitboard{ attacks };
		}
	}
}

Bitboard Attacks::knightAttacks(const Square square) noexcept
{
	return KnightAttacks[square];
}

Bitboard Attacks::bishopAttacks(const Square square, Bitboard blockers) noexcept
{
	blockers &= BishopMasks[square];
	const u64 key = (blockers.value() * BishopMagics[square]) >> (64u - BishopIndexBits[square]);
	return BishopAttacks[square][key];
}

Bitboard Attacks::rookAttacks(const Square square, Bitboard blockers) noexcept
{
	blockers &= RookMasks[square];
	const u64 key = (blockers.value() * RookMagics[square]) >> (64u - RookIndexBits[square]);
	return RookAttacks[square][key];
}

Bitboard Attacks::queenAttacks(const Square square, const Bitboard blockers) noexcept
{
	return bishopAttacks(square, blockers) | rookAttacks(square, blockers);
}

Bitboard Attacks::kingAttacks(const Square square) noexcept
{
	return KingAttacks[u8(square)];
}

Bitboard Attacks::pieceAttacks(const PieceType pieceType, const Square square, const Bitboard blockers) noexcept
{
	switch (pieceType)
	{
		case KNIGHT:
			return knightAttacks(square);
		case BISHOP:
			return bishopAttacks(square, blockers);
		case ROOK:
			return rookAttacks(square, blockers);
		case QUEEN:
			return queenAttacks(square, blockers);
		case KING:
			return kingAttacks(square);
		default:
			return {};
	}
}

Bitboard Attacks::bishopXRayAttacks(const Square square) noexcept
{
	return BishopXRayAttacks[u8(square)];
}

Bitboard Attacks::rookXRayAttacks(const Square square) noexcept
{
	return RookXRayAttacks[u8(square)];
}
