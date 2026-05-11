plugins {
    id("com.android.library") version "8.5.0"
    kotlin("android") version "1.9.24"
}

android {
    namespace = "io.jova.core"
    compileSdk = 36
    defaultConfig {
        minSdk = 24
        ndk { abiFilters += listOf("arm64-v8a", "armeabi-v7a", "x86_64", "x86") }
    }
    sourceSets {
        named("main") {
            jniLibs.srcDirs("src/main/jniLibs")
            kotlin.srcDirs("src/main/kotlin")
        }
        named("test") {
            kotlin.srcDirs("src/test/kotlin")
            resources.srcDirs("src/test/resources")
        }
    }
    testOptions {
        unitTests.isIncludeAndroidResources = true
        unitTests.all {
            // For JVM unit tests on macOS host: point JNA at the native dylib built for
            // aarch64-apple-darwin. Android slice validation is CI-only (requires a device/emulator).
            val repoRoot = rootProject.projectDir.parentFile.parentFile.absolutePath
            val dylibDir = "$repoRoot/target/aarch64-apple-darwin/release"
            it.jvmArgs("-Djna.library.path=$dylibDir")
        }
    }
}

kotlin {
    jvmToolchain(21)
}

dependencies {
    implementation("net.java.dev.jna:jna:5.14.0@aar")
    testImplementation("net.java.dev.jna:jna:5.14.0")
    testImplementation("junit:junit:4.13.2")
    testImplementation("org.json:json:20240303")
}
