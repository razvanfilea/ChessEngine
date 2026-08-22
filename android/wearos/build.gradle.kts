plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlinAndroid)
    alias(libs.plugins.compose.compiler)
}

android {
    namespace = "net.theluckycoder.chess.wearos"
    compileSdk = Versions.Sdk.compile

    defaultConfig {
        applicationId = "net.theluckycoder.chess"
        minSdk = Versions.Sdk.wearOsMin
        targetSdk = Versions.Sdk.target
        versionCode = Versions.App.code
        versionName = Versions.App.name
        resourceConfigurations += listOf("en")
    }

    buildTypes {
        release {
            isMinifyEnabled = true
            isShrinkResources = true
            proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"), "proguard-rules.pro")

            packagingOptions {
                resources {
                    excludes.add("DebugProbesKt.bin")
                }
            }
        }
    }

    buildFeatures.compose = true

    composeOptions {
        kotlinCompilerExtensionVersion = Versions.composeCompiler
    }
}

tasks.withType(org.jetbrains.kotlin.gradle.tasks.KotlinCompile::class).configureEach {
    kotlinOptions {
        freeCompilerArgs = listOf(
            "-Xopt-in=kotlin.RequiresOptIn",
        )
    }
}

dependencies {
    implementation(project(path = ":common"))

    implementation("androidx.wear.compose:compose-foundation:1.1.0-rc01")
    implementation("androidx.wear.compose:compose-material:1.1.0-rc01")
}
