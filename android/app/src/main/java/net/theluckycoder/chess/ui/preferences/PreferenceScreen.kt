package net.theluckycoder.chess.ui.preferences

import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.datastore.core.DataStore
import androidx.datastore.preferences.core.Preferences
import androidx.datastore.preferences.core.edit
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch

@Composable
fun PreferenceScreen(
    dataStore: DataStore<Preferences>,
    items: List<BasePreferenceItem>
) {
    val scope = rememberCoroutineScope()
    val prefs by dataStore.data.collectAsState(initial = null)

    LazyColumn(modifier = Modifier.fillMaxSize()) {
        items(items = items) { item ->
            MatchPreferenceItem(scope, dataStore, prefs, item)
        }
    }
}

@Composable
private fun MatchPreferenceItem(
    scope: CoroutineScope,
    dataStore: DataStore<Preferences>,
    prefs: Preferences?,
    item: BasePreferenceItem
) {
    if (item is PreferenceItem) {
        val dependencyKey = item.dependencyKey
        if (dependencyKey != null && prefs?.get(dependencyKey) != true)
            return
    }

    when (item) {
        is PreferenceGroupItem -> {
            Text(
                text = item.title,
                fontSize = 14.sp,
                color = MaterialTheme.colorScheme.secondary,
                fontWeight = FontWeight.Medium,
                modifier = Modifier.padding(start = 72.dp, top = 24.dp, bottom = 8.dp)
            )

            item.items.forEach { child ->
                MatchPreferenceItem(scope, dataStore, prefs, child)
            }
        }
        is EmptyPreferenceItem -> {
            Preference(item = item, onClick = item.onClick)
        }
        is SwitchPreferenceItem -> {
            SwitchPreference(
                item = item,
                value = prefs?.get(item.prefKey),
                onValueChanged = { newValue ->
                    scope.launch(Dispatchers.IO) {
                        dataStore.edit { it[item.prefKey] = newValue }
                    }
                }
            )
        }
        is SeekbarFloatPreferenceItem -> {
            SeekBarFloatPreference(
                item = item,
                value = prefs?.get(item.prefKey),
                onValueChanged = { newValue ->
                    scope.launch {
                        dataStore.edit { it[item.prefKey] = newValue }
                    }
                },
            )
        }
        is SeekbarIntPreferenceItem -> {
            SeekBarIntPreference(
                item = item,
                value = prefs?.get(item.prefKey),
                onValueChanged = { newValue ->
                    scope.launch {
                        dataStore.edit { it[item.prefKey] = newValue }
                    }
                },
            )
        }
    }
}
