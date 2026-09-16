package net.theluckycoder.chess

import android.app.Application
import kotlinx.coroutines.DelicateCoroutinesApi
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch
import net.theluckycoder.chess.common.SettingsDataStore
import net.theluckycoder.chess.common.model.SearchOptions
import kotlin.time.Duration.Companion.seconds

@Suppress("unused")
class ChessApp : Application() {

    @OptIn(DelicateCoroutinesApi::class)
    override fun onCreate() {
        super.onCreate()

        GlobalScope.launch(Dispatchers.IO) {
            val dataStore = SettingsDataStore.get(this@ChessApp)

            launch {
                if (dataStore.firstStart().first()) {
                    val engineSettings = SearchOptions.DEFAULT
                        .copy(searchTime = SettingsDataStore.DEFAULT_SEARCH_TIME.seconds)
                    dataStore.setEngineSettings(engineSettings)
                    dataStore.setFirstStart(false)
                }
            }
        }
    }

    companion object {
        init {
            System.loadLibrary("chess")
        }
    }
}
