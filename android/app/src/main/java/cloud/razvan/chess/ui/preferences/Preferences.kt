package cloud.razvan.chess.ui.preferences

import androidx.annotation.DrawableRes
import androidx.annotation.StringRes
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material3.Icon
import androidx.compose.material3.ListItem
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Slider
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlin.math.roundToInt

@Composable
fun PreferenceCategory(@StringRes title: Int) {
    Text(
        text = stringResource(title),
        fontSize = 14.sp,
        color = MaterialTheme.colorScheme.secondary,
        fontWeight = FontWeight.Medium,
        modifier = Modifier.padding(start = 72.dp, top = 24.dp, bottom = 8.dp)
    )
}

@Composable
fun Preference(
    title: String,
    summary: String,
    @DrawableRes icon: Int,
    onClick: (() -> Unit)? = null,
    trailing: @Composable (() -> Unit)? = null,
    content: @Composable () -> Unit = {},
) {
    ListItem(
        headlineContent = { Text(title) },
        supportingContent = {
            Column {
                Text(summary)
                content()
            }
        },
        leadingContent = {
            Icon(
                painter = painterResource(icon),
                contentDescription = null,
                modifier = Modifier
                    .padding(8.dp)
                    .size(24.dp)
            )
        },
        trailingContent = trailing,
        modifier = if (onClick != null) Modifier.clickable(onClick = onClick) else Modifier,
    )
}

@Composable
fun SwitchPreference(
    @StringRes title: Int,
    @StringRes summary: Int,
    @DrawableRes icon: Int,
    checked: Boolean,
    onCheckedChange: (Boolean) -> Unit,
) {
    Preference(
        title = stringResource(title),
        summary = stringResource(summary),
        icon = icon,
        onClick = { onCheckedChange(!checked) },
        trailing = { Switch(checked = checked, onCheckedChange = onCheckedChange) },
    )
}

@Composable
fun SliderPreference(
    @StringRes title: Int,
    @StringRes summary: Int,
    @DrawableRes icon: Int,
    value: Int,
    range: IntRange,
    onValueChange: (Int) -> Unit,
) {
    var current by remember(value) { mutableIntStateOf(value) }

    Preference(title = stringResource(title), summary = stringResource(summary), icon = icon) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Text(
                text = current.toString(),
                color = MaterialTheme.colorScheme.onBackground,
                modifier = Modifier.width(32.dp),
            )
            Slider(
                value = current.toFloat(),
                onValueChange = { current = it.roundToInt() },
                valueRange = range.first.toFloat()..range.last.toFloat(),
                onValueChangeFinished = { onValueChange(current) },
            )
        }
    }
}
