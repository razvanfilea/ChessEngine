package net.theluckycoder.chess.common

import android.annotation.SuppressLint
import net.theluckycoder.chess.common.model.GameState
import net.theluckycoder.chess.common.model.Move
import java.text.SimpleDateFormat
import java.util.*

object Pgn {

    private fun StringBuilder.tag(name: String, value: String) {
        append('[').append(name).append(" \"").append(value).append("\"]\n")
    }

    private fun StringBuilder.resultTag(gameState: GameState): String {
        val value = when (gameState) {
            GameState.WINNER_WHITE -> "1-0"
            GameState.WINNER_BLACK -> "0-1"
            GameState.DRAW -> "1/2-1/2"
            else -> "*"
        }

        tag("Result", value)
        return value
    }

    private fun StringBuilder.exportMoves(moves: List<Move>) {
        append('\n')
        moves.forEachIndexed { index, move ->
            if (index % 2 == 0) {
                append(index / 2 + 1)
                append('.')
                append(' ')
            }

            append(move.toString())
            append(' ')
        }
    }

    @SuppressLint("SimpleDateFormat")
    fun export(
        isPlayerWhite: Boolean,
        startFen: String,
        moves: List<Move>,
        gameState: GameState
    ): String {
        return buildString {
            tag("Event", "Casual Game")
            tag("Site", "http://theluckycoder.net/apps/chess")
            tag("Date", SimpleDateFormat("yyyy.MM.dd").format(Date()))

            val (white, black) = if (isPlayerWhite) PLAYER to ENGINE else ENGINE to PLAYER
            tag("White", white)
            tag("Black", black)

            tag("FEN", startFen)
            tag("PlyCount", moves.size.toString())
            val result = resultTag(gameState)

            exportMoves(moves)
            append(result)
        }
    }

    private const val PLAYER = "Player"
    private const val ENGINE = "Lucky Chess"
}
