rm -rf ./jniLibs

echo "Building for Android x86_64, armv7, aarch64:"

export  ANDROID_NDK=r25b ANDROID_SDK=26 ANDROID_VERSION=11.0.0_r48 && cross build --target x86_64-linux-android --release --lib && \
  cross build --target armv7-linux-androideabi --release --lib && \
  cross build --target aarch64-linux-android --release --lib

echo "Copying results to jniLibs"

mkdir -p jniLibs/arm64-v8a/ && \
    cp target/aarch64-linux-android/release/libcelandroid.so jniLibs/arm64-v8a/libuniffi_celandroid.so && \
mkdir -p jniLibs/armeabi-v7a/ && \
    cp target/armv7-linux-androideabi/release/libcelandroid.so jniLibs/armeabi-v7a/libuniffi_celandroid.so && \
mkdir -p jniLibs/x86_64/ && \
    cp target/x86_64-linux-android/release/libcelandroid.so jniLibs/x86_64/libuniffi_celandroid.so

echo "Running UniFFI to generate kotlin bindings"

cargo run --features=uniffi/cli \
    --bin uniffi-bindgen \
    generate src/cel.udl \
    --language kotlin

echo "Copying build results to /target/android"
# Define the source directories and file
src_dir_jniLibs="jniLibs"
src_file_cel="src/uniffi/cel/cel.kt"

# Define the target directory
target_dir="target/android"

# Create the target directories if they do not exist
mkdir -p "${target_dir}/jniLibs"
mkdir -p "${target_dir}/java/uniffi/cel"

# Copy the generated SO files target directory
cp -R "${src_dir_jniLibs}/." "${target_dir}/jniLibs/"
for target_file in $(find ${target_dir} -type f -name 'libuniffi_celandroid.so'); do
    # Replace celandroid with cel in the target file path
    new_file="${target_file/celandroid/cel}"
    # Rename the target file
    mv "${target_file}" "${new_file}"
done

# Copy the generated Kotlin interface to the target directory
cp "${src_file_cel}" "${target_dir}/java/uniffi/cel/"


echo "Build done --- ✅ "
printf "\e]8;;file://$(pwd)/target/android/\aFind your output in ./target/android/\n"