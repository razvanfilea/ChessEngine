package net.theluckycoder.chess.common

import android.app.Application
import android.content.Context
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import net.theluckycoder.chess.common.cpp.Native
import net.theluckycoder.chess.common.model.Move
import java.io.File
import java.io.FileNotFoundException

object SaveManager {

    private const val SAVE_FILE_NAME = "moves.save"

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

    fun loadFromFile(context: Context): Boolean = try {
        context.openFileInput(SAVE_FILE_NAME).bufferedReader().use { reader ->
            val lines = reader.readLines().toMutableList()

            val fen = lines.removeFirst()
            val playerWhite = lines.removeFirst().toInt() == 1

            val moves = lines.map { it.toInt() }

            return if (fen.isNotBlank() && moves.isNotEmpty()) {
                Native.loadFenMoves(playerWhite, fen, moves.toIntArray())
                true
            } else false
        }
    } catch (e: FileNotFoundException) {
        false
    }
}
