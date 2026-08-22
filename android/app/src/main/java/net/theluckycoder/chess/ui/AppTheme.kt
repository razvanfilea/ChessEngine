package net.theluckycoder.chess.ui

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color

val primaryDark = Color(0xFFA0D39A)
val onPrimaryDark = Color(0xFF093910)
val primaryContainerDark = Color(0xFF235024)
val onPrimaryContainerDark = Color(0xFFBCF0B5)
val secondaryDark = Color(0xFFD4C871)
val onSecondaryDark = Color(0xFF363100)
val secondaryContainerDark = Color(0xFF4F4800)
val onSecondaryContainerDark = Color(0xFFF1E58A)
val tertiaryDark = Color(0xFFA1CFD4)
val onTertiaryDark = Color(0xFF00363B)
val tertiaryContainerDark = Color(0xFF1F4D52)
val onTertiaryContainerDark = Color(0xFFBCEBF1)
val errorDark = Color(0xFFFFB4AB)
val onErrorDark = Color(0xFF690005)
val errorContainerDark = Color(0xFF93000A)
val onErrorContainerDark = Color(0xFFFFDAD6)
val backgroundDark = Color(0xFF10140F)
val onBackgroundDark = Color(0xFFE0E4DB)
val surfaceDark = Color(0xFF10140F)
val onSurfaceDark = Color(0xFFE0E4DB)
val surfaceVariantDark = Color(0xFF424940)
val onSurfaceVariantDark = Color(0xFFC2C9BD)
val outlineDark = Color(0xFF8C9388)
val outlineVariantDark = Color(0xFF424940)
val scrimDark = Color(0xFF000000)
val inverseSurfaceDark = Color(0xFFE0E4DB)
val inverseOnSurfaceDark = Color(0xFF2D322C)
val inversePrimaryDark = Color(0xFF3B693A)
val surfaceDimDark = Color(0xFF10140F)
val surfaceBrightDark = Color(0xFF363A34)
val surfaceContainerLowestDark = Color(0xFF0B0F0A)
val surfaceContainerLowDark = Color(0xFF181D17)
val surfaceContainerDark = Color(0xFF1C211B)
val surfaceContainerHighDark = Color(0xFF272B25)
val surfaceContainerHighestDark = Color(0xFF323630)

private val darkScheme = darkColorScheme(
    primary = primaryDark,
    onPrimary = onPrimaryDark,
    primaryContainer = primaryContainerDark,
    onPrimaryContainer = onPrimaryContainerDark,
    secondary = secondaryDark,
    onSecondary = onSecondaryDark,
    secondaryContainer = secondaryContainerDark,
    onSecondaryContainer = onSecondaryContainerDark,
    tertiary = tertiaryDark,
    onTertiary = onTertiaryDark,
    tertiaryContainer = tertiaryContainerDark,
    onTertiaryContainer = onTertiaryContainerDark,
    error = errorDark,
    onError = onErrorDark,
    errorContainer = errorContainerDark,
    onErrorContainer = onErrorContainerDark,
    background = backgroundDark,
    onBackground = onBackgroundDark,
    surface = surfaceDark,
    onSurface = onSurfaceDark,
    surfaceVariant = surfaceVariantDark,
    onSurfaceVariant = onSurfaceVariantDark,
    outline = outlineDark,
    outlineVariant = outlineVariantDark,
    scrim = scrimDark,
    inverseSurface = inverseSurfaceDark,
    inverseOnSurface = inverseOnSurfaceDark,
    inversePrimary = inversePrimaryDark,
    surfaceDim = surfaceDimDark,
    surfaceBright = surfaceBrightDark,
    surfaceContainerLowest = surfaceContainerLowestDark,
    surfaceContainerLow = surfaceContainerLowDark,
    surfaceContainer = surfaceContainerDark,
    surfaceContainerHigh = surfaceContainerHighDark,
    surfaceContainerHighest = surfaceContainerHighestDark,
)

@Composable
fun ChessMaterialTheme(content: @Composable () -> Unit) {
    MaterialTheme(
        colorScheme = darkScheme,
        content = content
    )
}