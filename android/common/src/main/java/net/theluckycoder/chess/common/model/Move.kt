package net.theluckycoder.chess.common.model

import androidx.annotation.Keep

@Keep
class Move(val content: Int) {
    val from: Byte get() = (content and 0x3F).toByte()
    val to: Byte get() = ((content shr 6) and 0x3F).toByte()
    val flagsBits: Int get() = (content shr 12) and 0x0F

    val isCapture: Boolean get() = (flagsBits and 0b0100) != 0
    val isPromotion: Boolean get() = (flagsBits and 0b1000) != 0
    val isCastleKing: Boolean get() = flagsBits == 2
    val isCastleQueen: Boolean get() = flagsBits == 3
    val isCastle: Boolean get() = isCastleKing || isCastleQueen
    val isEnPassant: Boolean get() = flagsBits == 5

    // In chess_core: (flags & 0b0011): 0->Knight, 1->Bishop, 2->Rook, 3->Queen
    val promotedPieceType: Byte get() = if (isPromotion) {
        when (flagsBits and 0b0011) {
            0 -> Piece.KNIGHT
            1 -> Piece.BISHOP
            2 -> Piece.ROOK
            3 -> Piece.QUEEN
            else -> Piece.NONE
        }
    } else Piece.NONE

    class Flags(private val move: Move) {
        val capture: Boolean get() = move.isCapture
        val promotion: Boolean get() = move.isPromotion
        val kSideCastle: Boolean get() = move.isCastleKing
        val qSideCastle: Boolean get() = move.isCastleQueen
        val enPassant: Boolean get() = move.isEnPassant
    }

    val flags: Flags get() = Flags(this)

    override fun equals(other: Any?): Boolean = other is Move && content == other.content
    override fun hashCode(): Int = content

    override fun toString(): String {
        if (isCastleKing) return "0-0"
        if (isCastleQueen) return "0-0-0"

        val fromFile = ('a' + (from % 8))
        val fromRank = ('1' + (from / 8))
        val toFile = ('a' + (to % 8))
        val toRank = ('1' + (to / 8))

        val promo = if (isPromotion) {
            when (promotedPieceType) {
                Piece.QUEEN -> "=Q"
                Piece.ROOK -> "=R"
                Piece.BISHOP -> "=B"
                Piece.KNIGHT -> "=N"
                else -> ""
            }
        } else ""

        val sep = if (isCapture) "x" else "-"
        return "$fromFile$fromRank$sep$toFile$toRank$promo"
    }
}
