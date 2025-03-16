#!/bin/bash

set -euxo pipefail

if [[ -z "$ANDROID_HOME" ]] then 
    echo ANDROID_HOME is not set!
    exit 1
fi

rm -rf build
mkdir build
javac -cp "$ANDROID_HOME"/platforms/android-33/android.jar -source 1.8 -target 1.8 java/*.java -d build
jar cvf build/bluest.jar -C build .
java -classpath "$ANDROID_HOME"/build-tools/35.0.1/lib/d8.jar com.android.tools.r8.D8 --lib "$ANDROID_HOME"/platforms/android-33/android.jar --min-api 20 build/bluest.jar
java-spaghetti-gen generate --verbose
rm -rf build
rustfmt bindings.rs
