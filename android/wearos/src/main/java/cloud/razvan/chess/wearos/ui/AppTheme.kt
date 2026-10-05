package cloud.razvan.chess.wearos.ui

import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.lerp
import androidx.wear.compose.material3.ColorScheme
import androidx.wear.compose.material3.MaterialTheme
import cloud.razvan.chess.common.ui.*

private fun Color.dim() = lerp(this, Color.Black, 0.25f)

private val WearColorScheme = ColorScheme(
    primary = primaryDark,
    primaryDim = primaryDark.dim(),
    primaryContainer = primaryContainerDark,
    onPrimary = onPrimaryDark,
    onPrimaryContainer = onPrimaryContainerDark,
    secondary = secondaryDark,
    secondaryDim = secondaryDark.dim(),
    secondaryContainer = secondaryContainerDark,
    onSecondary = onSecondaryDark,
    onSecondaryContainer = onSecondaryContainerDark,
    tertiary = tertiaryDark,
    tertiaryDim = tertiaryDark.dim(),
    tertiaryContainer = tertiaryContainerDark,
    onTertiary = onTertiaryDark,
    onTertiaryContainer = onTertiaryContainerDark,
    surfaceContainerLow = surfaceContainerLowDark,
    surfaceContainer = surfaceContainerDark,
    surfaceContainerHigh = surfaceContainerHighDark,
    onSurface = onSurfaceDark,
    onSurfaceVariant = onSurfaceVariantDark,
    outline = outlineDark,
    outlineVariant = outlineVariantDark,
    background = Color.Black,
    onBackground = onBackgroundDark,
    error = errorDark,
    errorDim = errorDark.dim(),
    errorContainer = errorContainerDark,
    onError = onErrorDark,
    onErrorContainer = onErrorContainerDark,
)

@Composable
fun ChessMaterialTheme(content: @Composable () -> Unit) {
    MaterialTheme(
        colorScheme = WearColorScheme,
        content = content
    )
}
