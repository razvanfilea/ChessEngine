package net.theluckycoder.chess.wearos.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.layout.Column
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.wear.compose.material.Button
import androidx.wear.compose.material.Text

@Composable
fun TextIconButton(
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    enabled: Boolean = true,
    text: String,
    icon: @Composable BoxScope.() -> Unit
) {
    Column(
        modifier = modifier,
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center
    ) {
        Button(
            enabled = enabled,
            onClick = onClick,
            content = icon
        )

        Text(text = text)
    }
}

@Composable
fun isScreenRound(): Boolean {
    val ctx = LocalContext.current
    return remember { ctx.resources.configuration.isScreenRound }
}
