rm -rf ./target/android

# Define the target directory
target_dir="target/android"

# Create the target directories if they do not exist
mkdir -p "${target_dir}"

echo "Building for Android x86_64, armv7, aarch64:"

for TARGET in \
        x86_64-linux-android \
        armv7-linux-androideabi \
        aarch64-linux-android
do
    rustup target add $TARGET
    export  ANDROID_NDK=r25b ANDROID_SDK=26 ANDROID_VERSION=11.0.0_r48 && cross build --target $TARGET --release --lib
    mkdir -p target/android/$TARGET/ && \
        cp target/$TARGET/release/libcel_eval.so target/android/$TARGET/libuniffi_celeval.so
done

export  ANDROID_NDK=r25b ANDROID_SDK=26 ANDROID_VERSION=11.0.0_r48 && cross build --target x86_64-linux-android --release --lib && \
  cross build --target armv7-linux-androideabi --release --lib && \
  cross build --target aarch64-linux-android --release --lib

echo "Running UniFFI to generate kotlin bindings"

cargo run --features=uniffi/cli \
    --bin uniffi-bindgen \
    generate src/cel.udl \
    --language kotlin \
    --out-dir ./target/android/

echo "Copying build results to /target/android"

# Copy the generated SO files target directory
for target_file in $(find ${target_dir} -type f -name 'libuniffi_cel_eval.so'); do
    # Replace celeval with cel in the target file path
    new_file="${target_file/celeval/cel}"
    # Rename the target file
    mv "${target_file}" "${new_file}"
done


echo "Build done --- ✅ "
printf "\e]8;;file://$(pwd)/target/android/\aFind your output in ./target/android/\n"