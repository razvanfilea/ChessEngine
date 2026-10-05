package cloud.razvan.chess.common.model

class IndexedPiece(val packed: Int) {
    val id: Int get() = packed and 0xFF
    val square: Int get() = (packed shr 8) and 0xFF
    val type: Byte get() = ((packed shr 16) and 0xFF).toByte()
    val isWhite: Boolean get() = ((packed shr 24) and 1) != 0

    val score: Int
        get() = when (type) {
            Piece.PAWN -> 1
            Piece.KNIGHT -> 3
            Piece.BISHOP -> 3
            Piece.ROOK -> 5
            Piece.QUEEN -> 9
            else -> 0
        }
}

object Piece {
    const val NONE: Byte = -1
    const val PAWN: Byte = 0
    const val KNIGHT: Byte = 1
    const val BISHOP: Byte = 2
    const val ROOK: Byte = 3
    const val QUEEN: Byte = 4
    const val KING: Byte = 5
}
