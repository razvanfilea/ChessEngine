package cloud.razvan.chess.common

import android.app.Application
import cloud.razvan.chess.common.model.Settings

open class ChessApplication : Application() {

    open val defaultSettings = Settings()

    companion object {
        init {
            System.loadLibrary("chess")
        }
    }
}
