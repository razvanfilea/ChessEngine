package net.theluckycoder.chess.wearos.ui

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.animation.ExperimentalAnimationApi
import net.theluckycoder.chess.wearos.ui.screen.WatchScreen

class WatchActivity : ComponentActivity() {

    @OptIn(ExperimentalAnimationApi::class)
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        setContent {
            ChessMaterialTheme {
                WatchScreen.Content()
            }
        }
    }
}
