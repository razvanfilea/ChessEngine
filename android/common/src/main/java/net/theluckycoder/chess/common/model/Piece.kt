package net.theluckycoder.chess.common.model

import androidx.annotation.Keep

@Keep // Used by native code
class IndexedPiece(
    val id: Int,
    private val square: Int,
    private val type: Byte,
    private val isWhite: Boolean
) {
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
        const val NONE: Byte = 0
        const val PAWN: Byte = 1
        const val KNIGHT: Byte = 2
        const val BISHOP: Byte = 3
        const val ROOK: Byte = 4
        const val QUEEN: Byte = 5
        const val KING: Byte = 6
    }
}
