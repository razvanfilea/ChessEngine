package net.theluckycoder.chess.common.model

import androidx.annotation.Keep
import net.theluckycoder.chess.common.toBoolean
import kotlin.experimental.and

@Keep
data class Move(
    val content: Int,
    val from: Byte,
    val to: Byte,
    val pieceType: Byte,
    val capturedPieceType: Byte,
    val promotedPieceType: Byte,
    private val internalFlags: Byte
) {
    companion object {
        const val CAPTURE: Byte = 1.toByte()
        const val PROMOTION: Byte = (1 shl 1).toByte()
        const val KSIDE_CASTLE: Byte = (1 shl 2).toByte()
        const val QSIDE_CASTLE: Byte = (1 shl 3).toByte()
        const val DOUBLE_PAWN_PUSH: Byte = (1 shl 4).toByte()
        const val EN_PASSANT: Byte = (1 shl 5).toByte()
    }

    @Suppress("unused")
    class Flags(flags: Int) {
        private val flags = (flags and 0x7F).toByte()

        val capture: Boolean
            get() = (flags and CAPTURE).toBoolean()

        val promotion: Boolean
            get() = (flags and PROMOTION).toBoolean()

        val kSideCastle: Boolean
            get() = (flags and KSIDE_CASTLE).toBoolean()

        val qSideCastle: Boolean
            get() = (flags and QSIDE_CASTLE).toBoolean()

        val doublePawnPush: Boolean
            get() = (flags and DOUBLE_PAWN_PUSH).toBoolean()

        val enPassant: Boolean
            get() = (flags and EN_PASSANT).toBoolean()
    }

    val flags = Flags(internalFlags.toInt())

    override fun toString(): String = buildString {
        when {
            flags.kSideCastle -> append("0-0")
            flags.qSideCastle -> append("0-0-0")
            else -> {
                val piece = getPieceChar(pieceType)

                if (piece != ' ')
                    append(piece)

                if (flags.enPassant) {
                    append('a' + from % 8)
                    append('x')
                }

                if (flags.capture)
                    append('x')

                val x = 'a' + to % 8
                val y = '1' + to / 8
                append(x)
                append(y)

                if (flags.promotion) {
                    append('=')
                    append(getPieceChar(promotedPieceType))
                    append('+')
                }
            }
        }
    }

    private fun getPieceChar(piece: Byte) = when (piece) {
        Piece.KNIGHT -> 'N'
        Piece.BISHOP -> 'B'
        Piece.ROOK -> 'R'
        Piece.QUEEN -> 'Q'
        Piece.KING -> 'K'
        else -> ' '
    }
}
