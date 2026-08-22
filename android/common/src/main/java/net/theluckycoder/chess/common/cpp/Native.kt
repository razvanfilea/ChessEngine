package net.theluckycoder.chess.common.cpp

import net.theluckycoder.chess.common.model.IndexedPiece
import net.theluckycoder.chess.common.model.Move

object Native {

    external fun initBoard(boardChangeListener: BoardChangeListener, isPlayerWhite: Boolean)
    external fun initBook(bookPath: String)
    external fun enableBook(enable: Boolean)
    external fun setSearchListener(searchListener: SearchListener)

    external fun loadFen(playerWhite: Boolean, fen: String): Boolean
    external fun loadFenMoves(playerWhite: Boolean, fen: String, moves: IntArray)

    external fun isEngineBusy(): Boolean
    external fun isPlayerWhite(): Boolean
    external fun isPlayersTurn(): Boolean

    external fun getPieces(): Array<IndexedPiece>
    external fun getPossibleMoves(square: Byte): Array<Move>

    fun makeMove(move: Move) = makeMove(move.content)
    private external fun makeMove(move: Int)
    external fun makeEngineMove()

    external fun stopSearch()

    external fun undoMoves()
    external fun redoMoves()

    external fun getCurrentFen(): String
    external fun getStartFen(): String
    external fun getMovesHistory(): Array<Move>
    external fun getCurrentMoveIndex(): Int
}
