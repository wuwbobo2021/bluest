//! XXX: generate proxies by `java-spaghetti`: support non-Android platforms by JNI `DefineClass`;
//! otherwise do not load the Android dex data by itself but have a public method that initializes
//! the proxy class object `OnceLock` by a class loaded outside of the generated code.
//!
//! NOTE: It is important to have `Send + Sync` restrictions for generated proxy traits, because
//! instances of `Arc<Box<dyn SomeProxy>>` may be shared across threads unsafely.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::sync::{Arc, LazyLock, OnceLock};

use super::bindings::android::bluetooth::le::{ScanCallback, ScanResult};
use super::bindings::java::lang::{Class, ClassLoader, String as JString, Throwable};
use super::vm_context::new_dex_class_loader;
use java_spaghetti::{Arg, AsJValue, Env, Global, Local, Ref};

const DEX_DATA: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/android/classes.dex"));
static DEX_CLASS_LOADER: LazyLock<Global<ClassLoader>> = LazyLock::new(|| new_dex_class_loader(DEX_DATA));

pub trait ScanCallbackProxy: Send + Sync + 'static {
    fn onScanResult<'env>(
        &self,
        env: Env<'env>,
        this: Ref<'env, ScanCallback>,
        callback_type: i32,
        result: Option<Ref<'env, ScanResult>>,
    );
    fn onScanFailed<'env>(&self, env: Env<'env>, this: Ref<'env, ScanCallback>, error_code: i32);
}

pub trait ScanCallbackProxyBuild {
    fn new_rust_proxy<'env>(
        env: Env<'env>,
        proxy: Arc<Box<dyn ScanCallbackProxy>>,
    ) -> Result<Local<'env, ScanCallback>, Local<'env, Throwable>>;
    fn rust_proxy_class(env: Env<'_>) -> &'static Global<Class>;
}

impl ScanCallbackProxyBuild for ScanCallback {
    fn new_rust_proxy<'env>(
        env: Env<'env>,
        proxy: Arc<Box<dyn ScanCallbackProxy>>,
    ) -> Result<Local<'env, ScanCallback>, Local<'env, Throwable>> {
        let proxy_jclass = Self::rust_proxy_class(env).as_raw();
        let raw_arc = Arc::into_raw(proxy) as i64;
        unsafe {
            let jni_args = [AsJValue::as_jvalue(&raw_arc)];
            let jni_method = env.require_method(proxy_jclass, "<init>\0", "(J)V\0");
            env.new_object_a(proxy_jclass, jni_method, jni_args.as_ptr())
        }
    }

    fn rust_proxy_class(env: Env<'_>) -> &'static Global<Class> {
        static CLASS: OnceLock<Global<Class>> = OnceLock::new();
        CLASS.get_or_init(|| {
            let class_loader = DEX_CLASS_LOADER.as_ref(env);
            let class_jstring = JString::from_env_str(env, "com.github.alexmoon.bluest.android.ScanCallbackProxy");
            let class_object = class_loader.loadClass(class_jstring).unwrap().unwrap().as_global();

            unsafe {
                let (mut name_result, mut sig_result) =
                    (*b"nativeOnScanResult\0", *b"(JILandroid/bluetooth/le/ScanResult;)V\0");
                let (mut name_failed, mut sig_failed) = (*b"nativeOnScanFailed\0", *b"(JI)V\0");
                let (mut name_finalize, mut sig_finalize) = (*b"nativeFinalize\0", *b"(J)V\0");
                let mut native_methods = [
                    java_spaghetti::sys::JNINativeMethod {
                        name: name_result.as_mut_ptr(),
                        signature: sig_result.as_mut_ptr(),
                        fnPtr: Java_com_github_alexmoon_bluest_android_ScanCallbackProxy_nativeOnScanResult as *mut _,
                    },
                    java_spaghetti::sys::JNINativeMethod {
                        name: name_failed.as_mut_ptr(),
                        signature: sig_failed.as_mut_ptr(),
                        fnPtr: Java_com_github_alexmoon_bluest_android_ScanCallbackProxy_nativeOnScanFailed as *mut _,
                    },
                    java_spaghetti::sys::JNINativeMethod {
                        name: name_finalize.as_mut_ptr(),
                        signature: sig_finalize.as_mut_ptr(),
                        fnPtr: Java_com_github_alexmoon_bluest_android_ScanCallbackProxy_nativeFinalize as *mut _,
                    },
                ];
                ((**env.as_raw()).v1_2.RegisterNatives)(
                    env.as_raw(),
                    class_object.as_raw(),
                    native_methods.as_mut_ptr(),
                    3,
                );
            }

            class_object
        })
    }
}

#[test]
fn arc_ptr_cast_32_bit_test() {
    // According to the Rust Reference, casting between two integers of the same size (e.g. i32 -> u32)
    // is a no-op, and behaviors of other casts between integers are platform-agnostic.
    assert_eq!(u32::MAX as i64 as u32, u32::MAX);
    assert_eq!(u32::MIN as i64 as u32, u32::MIN);
}

unsafe extern "system" fn Java_com_github_alexmoon_bluest_android_ScanCallbackProxy_nativeOnScanResult(
    env: Env<'_>,
    this: Arg<ScanCallback>,
    arc: i64,
    callbackType: i32,
    result: Arg<ScanResult>,
) {
    let arc: Arc<Box<dyn ScanCallbackProxy>> = Arc::from_raw(arc as usize as *const _);
    arc.onScanResult(env, this.into_ref(env).unwrap(), callbackType, result.into_ref(env));
    std::mem::forget(arc);
}

unsafe extern "system" fn Java_com_github_alexmoon_bluest_android_ScanCallbackProxy_nativeOnScanFailed(
    env: Env<'_>,
    this: Arg<ScanCallback>,
    arc: i64,
    errorCode: i32,
) {
    let arc: Arc<Box<dyn ScanCallbackProxy>> = Arc::from_raw(arc as usize as *const _);
    arc.onScanFailed(env, this.into_ref(env).unwrap(), errorCode);
    std::mem::forget(arc);
}

unsafe extern "system" fn Java_com_github_alexmoon_bluest_android_ScanCallbackProxy_nativeFinalize(
    _env: Env<'_>,
    _this: Arg<ScanCallback>,
    arc: i64,
) {
    let _: Arc<Box<dyn ScanCallbackProxy>> = Arc::from_raw(arc as usize as *const _);
}
