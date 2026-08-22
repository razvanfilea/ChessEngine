package net.theluckycoder.chess.common

import net.theluckycoder.chess.common.model.Piece

class CapturedPieces(
    val whiteScore: Int,
    val blackScore: Int,
    val capturedByWhite: List<Byte>,
    val capturedByBlack: List<Byte>,
) {

    companion object {
        fun from(pieces: List<Piece>): CapturedPieces {
            val (whitePieces, blackPieces) = pieces.partition { it.isWhite }

            val totalWhiteScore = whitePieces.sumOf { it.score }
            val totalBlackScore = blackPieces.sumOf { it.score }

            val whiteScore = (totalWhiteScore - totalBlackScore).coerceAtLeast(0)
            val blackScore = (totalBlackScore - totalWhiteScore).coerceAtLeast(0)

            val remainingWhite = whitePieces.map { it.type }.toMutableList()
            val remainingBlack = blackPieces.map { it.type }.toMutableList()

            for (whitePiece in whitePieces) {
                if (remainingBlack.contains(whitePiece.type)) {
                    remainingWhite.remove(whitePiece.type)
                    remainingBlack.remove(whitePiece.type)
                }
            }

            remainingWhite.sortByDescending { it }
            remainingBlack.sortByDescending { it }

            return CapturedPieces(
                whiteScore = whiteScore,
                blackScore = blackScore,
                capturedByWhite = remainingBlack,
                capturedByBlack = remainingWhite,
            )
        }
    }
}
