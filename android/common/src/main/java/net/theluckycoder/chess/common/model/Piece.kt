package net.theluckycoder.chess.common.model

import androidx.annotation.Keep

@Keep
class IndexedPiece(val packed: Int) {
    val id: Int get() = packed and 0xFF
    val square: Int get() = (packed shr 8) and 0xFF
    val type: Byte get() = ((packed shr 16) and 0xFF).toByte()
    val isWhite: Boolean get() = ((packed shr 24) and 1) != 0

    @Suppress("unused")
    constructor(
        id: Int,
        square: Int,
        type: Byte,
        isWhite: Boolean
    ) : this((id and 0xFF) or ((square and 0xFF) shl 8) or ((type.toInt() and 0xFF) shl 16) or (if (isWhite) 1 shl 24 else 0))

    fun toPiece() = Piece(square, type, isWhite)
}

data class Piece(
    val square: Int,
    val type: Byte,
    val isWhite: Boolean
) {

    val score: Int
        get() = when (type) {
            PAWN -> 1 // Pawn
            KNIGHT -> 3 // Knight
            BISHOP -> 3 // Bishop
            ROOK -> 5 // Rook
            QUEEN -> 9 // Queen
            else -> 0
        }

    companion object {
        const val NONE: Byte = -1
        const val PAWN: Byte = 0
        const val KNIGHT: Byte = 1
        const val BISHOP: Byte = 2
        const val ROOK: Byte = 3
        const val QUEEN: Byte = 4
        const val KING: Byte = 5
    }
}
