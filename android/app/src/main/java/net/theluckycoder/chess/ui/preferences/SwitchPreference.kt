package net.theluckycoder.chess.ui.preferences

import androidx.compose.material3.Switch
import androidx.compose.runtime.Composable

@Composable
fun SwitchPreference(
    item: SwitchPreferenceItem,
    value: Boolean?,
    onValueChanged: (Boolean) -> Unit
) {
    val currentValue = value ?: item.defaultValue
    Preference(
        item = item,
        onClick = { onValueChanged(!currentValue) }
    ) {
        Switch(
            checked = currentValue,
            onCheckedChange = { onValueChanged(!currentValue) },
        )
    }
}
