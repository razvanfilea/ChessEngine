package cloud.razvan.chess.common.ui

import android.os.Build
import android.os.VibrationEffect
import android.os.Vibrator
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.platform.LocalContext
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.compose.LocalLifecycleOwner
import androidx.lifecycle.repeatOnLifecycle
import cloud.razvan.chess.common.viewmodel.HomeViewModel

@Composable
fun VibrateOnEngineMove(viewModel: HomeViewModel) {
    val context = LocalContext.current
    val lifecycle = LocalLifecycleOwner.current.lifecycle

    LaunchedEffect(viewModel, lifecycle) {
        val vibrator = context.getSystemService(Vibrator::class.java) ?: return@LaunchedEffect
        lifecycle.repeatOnLifecycle(Lifecycle.State.STARTED) {
            viewModel.engineMoves.collect {
                when {
                    Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q ->
                        vibrator.vibrate(VibrationEffect.createPredefined(VibrationEffect.EFFECT_CLICK))
                    Build.VERSION.SDK_INT >= Build.VERSION_CODES.O ->
                        vibrator.vibrate(VibrationEffect.createOneShot(20, 80))
                    else -> @Suppress("DEPRECATION") vibrator.vibrate(20)
                }
            }
        }
    }
}
