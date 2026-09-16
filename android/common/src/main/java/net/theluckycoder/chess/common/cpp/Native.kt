package net.theluckycoder.chess.common.cpp

import androidx.annotation.Keep
import net.theluckycoder.chess.common.model.BoardState
import net.theluckycoder.chess.common.model.Move

@Keep
object Native {

    private val EMPTY_MOVES = IntArray(0)

    external fun initBoard(isPlayerWhite: Boolean = true): BoardState
    external fun loadFenMoves(fen: String, moves: IntArray = EMPTY_MOVES, isPlayerWhite: Boolean = true): BoardState?

    fun makeMove(move: Move): BoardState = makeMove(move.content)
    external fun makeMove(move: Int): BoardState

    external fun undo(): BoardState?
    external fun redo(): BoardState?

    external fun getBoardState(): BoardState
    external fun getPossibleMoves(square: Byte): IntArray
    external fun getCurrentFen(): String
    external fun getStartFen(): String

    external fun search(depth: Int, maxTimeMs: Long, hashSizeMb: Int, threadCount: Int): Int
    external fun stopSearch()
}
