import com.github.jengelman.gradle.plugins.shadow.tasks.ShadowJar

plugins {
    kotlin("jvm") version "2.3.0"
    id("com.gradleup.shadow") version "9.2.2"
    application
}

repositories {
    mavenCentral()
}

dependencies {
    implementation("com.github.javaparser:javaparser-symbol-solver-core:3.27.1")
}

java {
    toolchain {
        languageVersion = JavaLanguageVersion.of(21)
    }
}

application {
    mainClass = "net.hannesbraun.JavaGenericsAnalyzerKt"
}

tasks.withType<ShadowJar> {
    archiveFileName.set("${rootProject.name}.jar")
}
