package net.theluckycoder.chess.ui.preferences

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.painter.Painter
import androidx.compose.ui.unit.dp

@Composable
fun Preference(
    item: PreferenceItem,
    summary: String? = null, // TODO
    onClick: () -> Unit = { },
    trailing: @Composable (() -> Unit)? = null
) {
    ListItem(
        headlineContent = {
            Text(text = item.title, maxLines = if (item.singleLineTitle) 1 else Int.MAX_VALUE)
        },
        supportingContent = { Text(text = summary ?: item.summary) },
        leadingContent = { PreferenceIcon(painter = item.icon) },
        modifier = Modifier.clickable(onClick = onClick),
        trailingContent = trailing,
    )
}

@Composable
fun Preference(
    item: KeyPreferenceItem<*>,
    summary: @Composable () -> Unit,
    onClick: () -> Unit = { },
    trailing: @Composable (() -> Unit)? = null
) {
    ListItem(
        headlineContent = {
            Text(
                text = item.title,
                maxLines = if (item.singleLineTitle) 1 else Int.MAX_VALUE
            )
        },
        supportingContent = summary,
        leadingContent = { PreferenceIcon(painter = item.icon) },
        modifier = Modifier.clickable(onClick = onClick),
        trailingContent = trailing,
    )
}

@Composable
private fun PreferenceIcon(painter: Painter?) {
    val iconModifier = Modifier
        .padding(8.dp)
        .size(24.dp)

    if (painter != null) {
        Icon(
            painter = painter,
            contentDescription = null,
            modifier = iconModifier
        )
    } else {
        Spacer(modifier = iconModifier)
    }
}
