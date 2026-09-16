package net.theluckycoder.chess.common

import android.app.Application
import android.content.Context
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import net.theluckycoder.chess.common.cpp.Native
import net.theluckycoder.chess.common.model.BoardState
import net.theluckycoder.chess.common.model.Move
import java.io.File
import java.io.FileNotFoundException

object SaveManager {

    private const val SAVE_FILE_NAME = "moves.save"

    data class SavedGame(
        val state: BoardState,
        val playerWhite: Boolean,
    )

    suspend fun saveToFileAsync(
        application: Application,
        startFen: String,
        playerWhite: Boolean,
        moves: List<Move>
    ) {
        if (moves.isEmpty()) return

        val content = buildString {
            append(startFen)
            append('\n')
            append(if (playerWhite) 1 else 0)
            moves.forEach {
                append('\n').append(it.content.toString())
            }
        }

        withContext(Dispatchers.IO) { saveToFile(application, content) }
    }

    private fun saveToFile(context: Context, content: String) {
        val file = File(context.filesDir, SAVE_FILE_NAME)

        if (!file.exists())
            file.createNewFile()

        context.openFileOutput(SAVE_FILE_NAME, Context.MODE_PRIVATE).bufferedWriter().use {
            it.write(content)
        }
    }

    fun loadFromFile(context: Context): SavedGame? = try {
        context.openFileInput(SAVE_FILE_NAME).bufferedReader().use { reader ->
            val lines = reader.readLines().toMutableList()

            val fen = lines.removeFirstOrNull() ?: return null
            val playerWhite = (lines.removeFirstOrNull()?.toIntOrNull() ?: 1) == 1

            val moves = lines.mapNotNull { it.toIntOrNull() }

            if (fen.isNotBlank() && moves.isNotEmpty()) {
                val state = Native.loadFenMoves(fen, moves.toIntArray(), playerWhite)
                if (state != null) {
                    SavedGame(state, playerWhite)
                } else null
            } else null
        }
    } catch (e: Exception) {
        null
    }
}
