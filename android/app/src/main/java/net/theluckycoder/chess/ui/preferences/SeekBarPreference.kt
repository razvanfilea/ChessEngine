package net.theluckycoder.chess.ui.preferences

import androidx.compose.foundation.layout.*
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Slider
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import kotlin.math.roundToInt

@Composable
fun SeekBarFloatPreference(
    item: SeekbarFloatPreferenceItem,
    value: Float?,
    onValueChanged: (Float) -> Unit,
) {
    val currentValue = remember(value) { mutableStateOf(value ?: item.defaultValue) }
    Preference(
        item = item,
        summary = {
            PreferenceSummary(
                item = item,
                sliderValue = currentValue.value,
                onValueChanged = { currentValue.value = it },
                onValueChangeFinished = { onValueChanged(currentValue.value) }
            )
        },
    )
}

@Composable
fun SeekBarIntPreference(
    item: SeekbarIntPreferenceItem,
    value: Int?,
    onValueChanged: (Int) -> Unit,
) {
    val currentValue = remember(value) { mutableStateOf(value ?: item.defaultValue) }
    Preference(
        item = item,
        summary = {
            PreferenceSummary(
                item = item,
                sliderValue = currentValue.value,
                onValueChanged = { currentValue.value = it.roundToInt() },
                onValueChangeFinished = { onValueChanged(currentValue.value) }
            )
        },
    )
}

@Composable
private fun <T : Number> PreferenceSummary(
    item: SeekbarBasePreferenceItem<T>,
    sliderValue: T,
    onValueChanged: (Float) -> Unit,
    onValueChangeFinished: () -> Unit,
) {
    Column(modifier = Modifier.fillMaxWidth()) {
        Text(text = item.summary)
        Row(
            modifier = Modifier.fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Box(modifier = Modifier.width(32.dp)) {
                Text(
                    text = item.valueRepresentation(sliderValue),
                    color = MaterialTheme.colorScheme.onBackground,
                )
            }
            Slider(
                value = sliderValue.toFloat(),
                onValueChange = onValueChanged,
                valueRange = item.valueRange,
                steps = item.steps,
                onValueChangeFinished = onValueChangeFinished
            )
        }
    }
}
