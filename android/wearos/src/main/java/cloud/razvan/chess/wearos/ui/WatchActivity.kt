package cloud.razvan.chess.wearos.ui

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.wear.compose.material3.AppScaffold
import cloud.razvan.chess.wearos.ui.screen.WatchScreen

class WatchActivity : ComponentActivity() {

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        setContent {
            ChessMaterialTheme {
                AppScaffold {
                    WatchScreen.Content()
                }
            }
        }
    }
}
