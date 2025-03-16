//! XXX: migrate this function into `java-spaghetti` or a seperate helper crate.

use java_spaghetti::{Global, Ref};

use super::bindings::{
    android::{content::Context, os::Build_VERSION},
    dalvik::system::{DexClassLoader, InMemoryDexClassLoader},
    java::lang::{ClassLoader, String as JString},
    java::nio,
};
use super::vm::VM;

pub fn ndk_context_available() -> bool {
    let ndk_ctx = ndk_context::android_context();
    !ndk_ctx.vm().is_null() && !ndk_ctx.context().is_null()
}

pub fn get_vm() -> VM {
    let vm = ndk_context::android_context().vm();
    if vm.is_null() {
        panic!("ndk-context is unconfigured: null JVM pointer, check the glue crate.");
    }
    unsafe { VM::from_raw(vm.cast()) }
}

pub fn get_android_context() -> Global<Context> {
    let ctx = ndk_context::android_context().context();
    if ctx.is_null() {
        // XXX: use `android.app.ActivityThread.getApplication()` and print a warning instead
        panic!("ndk-context is unconfigured: null Android context pointer, check the glue crate.");
    }
    get_vm().with_env(|env| {
        let context = unsafe { Ref::<'_, Context>::from_raw(env, ctx.cast()) };
        context.as_global()
    })
}

pub fn new_dex_class_loader(dex_data: &[u8]) -> Global<ClassLoader> {
    let vm = get_vm();
    let context = get_android_context();
    vm.with_env(|env| {
        let context = context.as_ref(env);
        let context_loader = context.getClassLoader().unwrap();
        if Build_VERSION::SDK_INT(env) >= 26 {
            use java_spaghetti::PrimitiveArray;
            // Safety: casts `&[u8]` to `&[i8]`.
            let data = unsafe { std::slice::from_raw_parts(dex_data.as_ptr() as *const i8, dex_data.len()) };
            let byte_array = java_spaghetti::ByteArray::new_from(env, data);
            let dex_buffer = nio::ByteBuffer::wrap_byte_array(env, byte_array).unwrap();
            let dex_loader =
                InMemoryDexClassLoader::new_ByteBuffer_ClassLoader(env, dex_buffer, context_loader).unwrap();
            dex_loader.cast::<ClassLoader>().unwrap().as_global()
        } else {
            let cache_dir = context.getCacheDir().unwrap().unwrap();
            let path_string = cache_dir.getAbsolutePath().unwrap().unwrap().to_string_lossy();
            let cache_dir_path = std::path::PathBuf::from(path_string);

            let dex_file_path = cache_dir_path.join(env!("CARGO_CRATE_NAME").to_string() + ".dex");
            std::fs::write(&dex_file_path, dex_data).unwrap();

            let oats_dir_path = cache_dir_path.join("oats");
            let _ = std::fs::create_dir(&oats_dir_path);

            let dex_file_jstring = JString::from_env_str(env, dex_file_path.to_string_lossy().as_ref());
            let oats_dir_jstring = JString::from_env_str(env, oats_dir_path.to_string_lossy().as_ref());

            let dex_loader = DexClassLoader::new(
                env,
                &dex_file_jstring,
                &oats_dir_jstring,
                java_spaghetti::Null,
                &context_loader,
            )
            .unwrap();
            dex_loader.cast::<ClassLoader>().unwrap().as_global()
        }
    })
}
