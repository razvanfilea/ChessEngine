package cloud.razvan.chess.common

import androidx.annotation.Keep
import cloud.razvan.chess.common.model.BoardState

@Keep
object Native {

    external fun initBoard(): BoardState
    external fun loadFen(fen: String): BoardState?

    external fun loadGame(save: String): BoardState?
    external fun saveGame(): String

    external fun makeMove(move: Int): BoardState

    external fun undo(isPlayerWhite: Boolean): BoardState?
    external fun redo(isPlayerWhite: Boolean): BoardState?

    external fun getCurrentFen(): String
    external fun exportPgn(date: String, isPlayerWhite: Boolean): String

    external fun engineMove(depth: Int, timeMs: Long, hashMb: Int, threads: Int): BoardState?
    external fun stopSearch()
}
