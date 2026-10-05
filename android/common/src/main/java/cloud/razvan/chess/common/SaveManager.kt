package cloud.razvan.chess.common

import android.content.Context
import android.util.AtomicFile
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import cloud.razvan.chess.common.model.BoardState
import java.io.File
import java.io.IOException

object SaveManager {

    private const val SAVE_FILE_NAME = "moves.save"

    data class SavedGame(
        val state: BoardState,
        val playerWhite: Boolean,
    )

    private fun saveFile(context: Context) = AtomicFile(File(context.filesDir, SAVE_FILE_NAME))

    suspend fun saveToFile(context: Context, playerWhite: Boolean, game: String) {
        val content = (if (playerWhite) "1\n" else "0\n") + game

        withContext(Dispatchers.IO) {
            val file = saveFile(context)
            val stream = try {
                file.startWrite()
            } catch (_: IOException) {
                return@withContext
            }
            try {
                stream.write(content.toByteArray())
                file.finishWrite(stream)
            } catch (_: IOException) {
                file.failWrite(stream)
            }
        }
    }

    /** Returns null when there is no save file or it can't be read */
    fun loadFromFile(context: Context): SavedGame? {
        val content = try {
            saveFile(context).readFully().decodeToString()
        } catch (_: IOException) {
            return null
        }

        val side = content.substringBefore('\n')
        val playerWhite = when (side) {
            "1" -> true
            "0" -> false
            else -> return null
        }

        val state = Native.loadGame(content.substringAfter('\n', "")) ?: return null
        return SavedGame(state, playerWhite)
    }
}
