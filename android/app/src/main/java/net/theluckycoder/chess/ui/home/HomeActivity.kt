package net.theluckycoder.chess.ui.home

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.compose.ui.platform.ComposeView
import net.theluckycoder.chess.ui.ChessMaterialTheme

class HomeActivity : ComponentActivity() {

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val view = ComposeView(this).apply {
            setContent {
                ChessMaterialTheme {
                    HomeScreen()
                }
            }
        }

        setContentView(view)
    }
}
