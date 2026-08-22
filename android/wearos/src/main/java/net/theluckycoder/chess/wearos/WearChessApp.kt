package net.theluckycoder.chess.wearos

import android.app.Application
import kotlinx.coroutines.DelicateCoroutinesApi
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch
import net.theluckycoder.chess.common.SettingsDataStore
import net.theluckycoder.chess.common.model.SearchOptions
import kotlin.time.Duration.Companion.seconds
import kotlin.time.ExperimentalTime

class WearChessApp : Application() {

    @OptIn(ExperimentalTime::class, DelicateCoroutinesApi::class)
    override fun onCreate() {
        super.onCreate()

        GlobalScope.launch(Dispatchers.IO) {
            val dataStore = SettingsDataStore.get(this@WearChessApp)

            launch {
                if (dataStore.firstStart().first()) {
                    // Set the default Engine Settings from native code
                    val engineSettings = SearchOptions.getNativeSearchOptions()
                        .copy(
                            searchTime = SettingsDataStore.DEFAULT_SEARCH_TIME.seconds,
                            hashSize = 16
                        )
                    dataStore.setEngineSettings(engineSettings)
                    dataStore.setFirstStart(false)

                    // Screen is too small for coordinates
                    dataStore.setShowCoordinates(false)
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
