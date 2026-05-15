plugins {
    id("com.android.library") version "8.10.1"
    kotlin("android") version "1.9.24"
}

android {
    namespace = "io.jova.core"
    ndkVersion = "29.0.14206865"
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
            // For JVM unit tests on the host (macOS dev or Linux CI): point JNA at the
            // host library produced by `bindings/kotlin/scripts/build-aar.sh` (which runs
            // `cargo build -p jova-core-ffi --release` without `--target`, so output lands
            // at `target/release/`). JNA picks the correct extension automatically — .dylib
            // on macOS, .so on Linux. Android slice validation is CI-only (requires a
            // device/emulator).
            val repoRoot = rootProject.projectDir.parentFile.parentFile.absolutePath
            val libDir = "$repoRoot/target/release"
            it.jvmArgs("-Djna.library.path=$libDir")
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
