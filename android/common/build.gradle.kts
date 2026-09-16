plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.compose.compiler)
}

android {
    namespace = "net.theluckycoder.chess.common"
    compileSdk = Versions.Sdk.compile
    ndkVersion = "27.2.12479018"

    defaultConfig {
        minSdk = Versions.Sdk.min

        consumerProguardFiles("consumer-rules.pro")

        ndk {
            abiFilters += listOf("arm64-v8a", "x86_64")
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

    inputs.dir("${rootDir.parentFile}/chess_android/src")
    inputs.dir("${rootDir.parentFile}/chess_core/src")
    inputs.dir("${rootDir.parentFile}/chess_engine/src")
    inputs.file("${rootDir.parentFile}/Cargo.toml")
    inputs.file("${rootDir.parentFile}/Cargo.lock")
    outputs.dir("${projectDir}/src/main/jniLibs")

    val isRelease = gradle.startParameter.taskNames.any { it.contains("Release", ignoreCase = true) }
    val cargoCmd = mutableListOf(
        "cargo", "ndk",
        "-t", "arm64-v8a",
        "-t", "x86_64",
        "-o", "${projectDir}/src/main/jniLibs",
        "build",
        "-p", "chess_android"
    )
    if (isRelease) {
        cargoCmd.add("--release")
    }
    commandLine(cargoCmd)
}

tasks.matching { it.name.startsWith("merge") && it.name.endsWith("JniLibFolders") }.configureEach {
    dependsOn(buildRustTask)
}

dependencies {
    debugApi(libs.kotlin.reflect)
    api(libs.kotlin.coroutinesAndroid)

    // AndroidX
    api(libs.androidx.core)
    api(libs.datastorePreferences)
    api(libs.androidx.viewmodelCompose)

    // Compose
    api(libs.compose.compiler)
    api(libs.compose.ui)
    api(libs.compose.toolingPreview)
    debugApi(libs.compose.tooling)
    api(libs.compose.foundation)
    implementation(libs.compose.material3)
    api(libs.compose.animation)
    api(libs.compose.activity)
}
