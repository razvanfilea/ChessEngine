package net.theluckycoder.chess.wearos.ui

import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.wear.compose.material.Colors
import androidx.wear.compose.material.MaterialTheme
import net.theluckycoder.chess.common.ui.AppColors

private val WearColors = Colors(
    primary = AppColors.Primary,
    primaryVariant = AppColors.PrimaryVariant,
    secondary = AppColors.Secondary,
    onPrimary = Color.White,
    onSecondary = Color.Black
)

@Composable
fun ChessMaterialTheme(content: @Composable () -> Unit) {
    MaterialTheme(
        colors = WearColors,
        content = content
    )
}