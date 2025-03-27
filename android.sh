export BINDGEN_EXTRA_CLANG_ARGS="\
--target=x86_64-linux-android \
--sysroot=$ANDROID_NDK/toolchains/llvm/prebuilt/linux-x86_64/sysroot \
-I$ANDROID_NDK/toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/include \
-I$ANDROID_NDK/toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/include/x86_64-linux-android"
