package net.theluckycoder.chess

import android.app.Application
import android.content.Context
import kotlinx.coroutines.DelicateCoroutinesApi
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch
import net.theluckycoder.chess.common.SettingsDataStore
import net.theluckycoder.chess.common.cpp.Native
import net.theluckycoder.chess.common.model.SearchOptions
import java.io.File
import kotlin.time.Duration
import kotlin.time.Duration.Companion.seconds
import kotlin.time.ExperimentalTime

@Suppress("unused")
class ChessApp : Application() {

    @OptIn(DelicateCoroutinesApi::class)
    override fun onCreate() {
        super.onCreate()

        GlobalScope.launch(Dispatchers.IO) {
            val dataStore = SettingsDataStore.get(this@ChessApp)

            try {
                copyBook()
            } catch (e: Exception) {
                e.printStackTrace()
            }

            launch {
                if (dataStore.firstStart().first()) {
                    // Set the default Engine Settings from native code
                    val engineSettings = SearchOptions.getNativeSearchOptions()
                        .copy(searchTime = SettingsDataStore.DEFAULT_SEARCH_TIME.seconds)
                    dataStore.setEngineSettings(engineSettings)
                    dataStore.setFirstStart(false)
                }
            }
        }
    }

    private fun copyBook() {
        val dest = getBookPath(this)

        assets.open(BOOK_NAME).use { input ->
            dest.outputStream().use { output ->
                input.copyTo(output)
            }
        }

        Native.initBook(dest.absolutePath)
    }

    companion object {
        init {
            System.loadLibrary("chess")
        }

        const val BOOK_NAME = "OpeningBook.bin"

        fun getBookPath(context: Context): File = File(context.filesDir, BOOK_NAME)
    }
}
