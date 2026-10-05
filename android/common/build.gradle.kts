plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.compose.compiler)
}

android {
    namespace = "cloud.razvan.chess.common"
    compileSdk = Versions.Sdk.compile
    ndkVersion = "27.2.12479018"

    defaultConfig {
        minSdk = Versions.Sdk.min

        ndk {
            abiFilters += listOf("arm64-v8a", "armeabi-v7a")
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_21
        targetCompatibility = JavaVersion.VERSION_21
    }

    buildFeatures.compose = true
}

val buildRustTask = tasks.register<Exec>("buildRust") {
    val ndkProvider = androidComponents.sdkComponents.ndkDirectory
    workingDir = rootDir.parentFile

    doFirst {
        val ndkHome = ndkProvider.get().asFile.absolutePath
        environment("ANDROID_NDK_HOME", ndkHome)
    }
    // The workspace release profile keeps debug info for engine profiling; AGP can't strip it here
    environment("CARGO_PROFILE_RELEASE_STRIP", "symbols")

    inputs.dir("${rootDir.parentFile}/chess_android/src")
    inputs.dir("${rootDir.parentFile}/chess_core/src")
    inputs.dir("${rootDir.parentFile}/chess_engine/src")
    inputs.file("${rootDir.parentFile}/Cargo.toml")
    inputs.file("${rootDir.parentFile}/Cargo.lock")
    outputs.dir("${projectDir}/src/main/jniLibs")

    val cargoCmd = mutableListOf(
        "cargo", "ndk",
        "-t", "arm64-v8a",
        "-t", "armeabi-v7a",
        "-o", "${projectDir}/src/main/jniLibs",
        "build",
        "-p", "chess_android",
        "--release"
    )
    commandLine(cargoCmd)
}

tasks.matching { it.name.startsWith("merge") && it.name.endsWith("JniLibFolders") }.configureEach {
    dependsOn(buildRustTask)
}

dependencies {
    api(libs.kotlin.coroutinesAndroid)

    api(libs.androidx.core)
    api(libs.datastorePreferences)
    api(libs.androidx.viewmodelCompose)

    api(libs.compose.ui)
    api(libs.compose.toolingPreview)
    debugApi(libs.compose.tooling)
    api(libs.compose.foundation)
    api(libs.compose.animation)
    api(libs.compose.activity)
}
