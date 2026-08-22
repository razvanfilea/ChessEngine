plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlinAndroid)
    alias(libs.plugins.compose.compiler)
}

android {
    namespace = "net.theluckycoder.chess"
    compileSdk = Versions.Sdk.compile

    defaultConfig {
        applicationId = "net.theluckycoder.chess"
        minSdk = Versions.Sdk.min
        targetSdk = Versions.Sdk.target
        versionCode = Versions.App.code
        versionName = Versions.App.name
        resourceConfigurations += listOf("en")
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_21
        targetCompatibility = JavaVersion.VERSION_21
    }

    buildTypes {
        release {
            isMinifyEnabled = true
            isShrinkResources = true
            proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"), "proguard-rules.pro")
        }
    }

    buildFeatures.compose = true
}

dependencies {
    implementation(project(path = ":common"))

    implementation(libs.compose.material3)
}
