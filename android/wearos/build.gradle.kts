plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.compose.compiler)
}

android {
    namespace = "cloud.razvan.chess.wearos"
    compileSdk = Versions.Sdk.compile

    defaultConfig {
        applicationId = "cloud.razvan.chess.wearos"
        minSdk = Versions.Sdk.wearOsMin
        targetSdk = Versions.Sdk.target
        versionCode = Versions.App.code
        versionName = Versions.App.name
    }

    splits {
        abi {
            isEnable = true
            reset()
            include("arm64-v8a", "armeabi-v7a")
            isUniversalApk = false
        }
    }

    androidResources {
        localeFilters += listOf("en")
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_21
        targetCompatibility = JavaVersion.VERSION_21
    }

    buildTypes {
        release {
            isMinifyEnabled = true
            isShrinkResources = true
            proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"))
        }
    }

    packaging {
        resources {
            excludes.add("DebugProbesKt.bin")
        }
    }

    buildFeatures.compose = true
}

dependencies {
    implementation(project(path = ":common"))

    implementation(libs.wear.compose.foundation)
    implementation(libs.wear.compose.material3)
}
