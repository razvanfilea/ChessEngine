package cloud.razvan.chess.ui.home

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import cloud.razvan.chess.ui.ChessMaterialTheme

class HomeActivity : ComponentActivity() {

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            ChessMaterialTheme {
                HomeScreen()
            }
        }
    }
}
