package cloud.razvan.chess.common.model

/** [san] is only known for moves that were played, it is empty for possible moves */
class Move(val content: Int, val san: String = "") {
    val from: Byte get() = (content and 0x3F).toByte()
    val to: Byte get() = ((content shr 6) and 0x3F).toByte()
    val flagsBits: Int get() = (content shr 12) and 0x0F

    val isCapture: Boolean get() = (flagsBits and 0b0100) != 0
    val isPromotion: Boolean get() = (flagsBits and 0b1000) != 0

    val promotedPieceType: Byte get() = if (isPromotion) {
        when (flagsBits and 0b0011) {
            0 -> Piece.KNIGHT
            1 -> Piece.BISHOP
            2 -> Piece.ROOK
            3 -> Piece.QUEEN
            else -> Piece.NONE
        }
    } else Piece.NONE

    override fun equals(other: Any?): Boolean = other is Move && content == other.content
    override fun hashCode(): Int = content
}
