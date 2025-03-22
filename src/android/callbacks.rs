//! XXX: generate proxies by `java-spaghetti`: support non-Android platforms by JNI `DefineClass`;
//! otherwise do not load the Android dex data by itself but have a public method that initializes
//! the proxy class object `OnceLock` by a class loaded outside of the generated code.
//!
//! NOTE: It is important to have `Send + Sync` restrictions for generated proxy traits, because
//! instances of `Arc<Box<dyn SomeProxy>>` may be shared across threads unsafely.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused)]

use std::marker::PhantomData;
use std::sync::{Arc, LazyLock, OnceLock};

use super::bindings::android::bluetooth::le::{ScanCallback, ScanResult};
use super::bindings::android::bluetooth::{
    BluetoothGatt, BluetoothGattCallback, BluetoothGattCharacteristic, BluetoothGattDescriptor,
};
use super::bindings::android::os::Build_VERSION;
use super::bindings::java::lang::{Class, ClassLoader, String as JString, Throwable};
use super::vm_context::new_dex_class_loader;
use java_spaghetti::{
    sys::{jlong, jvalue},
    Arg, AsJValue, ByteArray, Env, Global, Local, Ref,
};

const DEX_DATA: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/android/classes.dex"));
static DEX_CLASS_LOADER: LazyLock<Global<ClassLoader>> = LazyLock::new(|| new_dex_class_loader(DEX_DATA));

#[repr(transparent)]
struct RawArc<T> {
    value: jlong,
    ph: PhantomData<T>,
}

impl<T: Send + Sync + 'static> RawArc<T> {
    pub fn from_arc(arc: Arc<T>) -> Self {
        Self {
            value: Arc::into_raw(arc) as jlong,
            ph: PhantomData,
        }
    }

    pub fn into_jlong_value(self) -> jvalue {
        jvalue { j: self.value }
    }

    pub unsafe fn clone_arc(&self) -> Arc<T> {
        let arc_self = Arc::<T>::from_raw(self.value as usize as *const _);
        let arc_owned = arc_self.clone();
        std::mem::forget(arc_self);
        arc_owned
    }

    pub unsafe fn destroy_arc(self) {
        let _ = Arc::<T>::from_raw(self.value as usize as *const _);
    }
}

#[test]
fn arc_ptr_cast_32_bit_test() {
    // According to the Rust Reference, casting between two integers of the same size (e.g. i32 -> u32)
    // is a no-op, and behaviors of other casts between integers are platform-agnostic.
    assert_eq!(u32::MAX as jlong as u32, u32::MAX);
    assert_eq!(u32::MIN as jlong as u32, u32::MIN);
}

pub trait ScanCallbackProxy: Send + Sync + 'static {
    fn onScanResult<'env>(
        &self,
        env: Env<'env>,
        this: Ref<'env, ScanCallback>,
        callback_type: i32,
        result: Option<Ref<'env, ScanResult>>,
    ) {
    }
    fn onScanFailed<'env>(&self, env: Env<'env>, this: Ref<'env, ScanCallback>, error_code: i32) {}
}

pub trait ScanCallbackProxyBuild {
    fn new_rust_proxy<'env, P: ScanCallbackProxy>(
        env: Env<'env>,
        proxy: P,
    ) -> Result<Local<'env, ScanCallback>, Local<'env, Throwable>>;
    fn rust_proxy_class(env: Env<'_>) -> &'static Global<Class>;
}

impl ScanCallbackProxyBuild for ScanCallback {
    fn new_rust_proxy<'env, P: ScanCallbackProxy>(
        env: Env<'env>,
        proxy: P,
    ) -> Result<Local<'env, ScanCallback>, Local<'env, Throwable>> {
        let arc = Arc::new(Box::new(proxy) as Box<dyn ScanCallbackProxy>);
        let proxy_jclass = Self::rust_proxy_class(env).as_raw();
        unsafe {
            let jni_args = [RawArc::from_arc(arc).into_jlong_value()];
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
                    native_methods.len() as i32,
                );
            }

            class_object
        })
    }
}

unsafe extern "system" fn Java_com_github_alexmoon_bluest_android_ScanCallbackProxy_nativeOnScanResult(
    env: Env<'_>,
    this: Arg<ScanCallback>,
    arc: RawArc<Box<dyn ScanCallbackProxy>>,
    callbackType: i32,
    result: Arg<ScanResult>,
) {
    let arc = arc.clone_arc();
    arc.onScanResult(env, this.into_ref(env).unwrap(), callbackType, result.into_ref(env));
}

unsafe extern "system" fn Java_com_github_alexmoon_bluest_android_ScanCallbackProxy_nativeOnScanFailed(
    env: Env<'_>,
    this: Arg<ScanCallback>,
    arc: RawArc<Box<dyn ScanCallbackProxy>>,
    errorCode: i32,
) {
    let arc = arc.clone_arc();
    arc.onScanFailed(env, this.into_ref(env).unwrap(), errorCode);
}

unsafe extern "system" fn Java_com_github_alexmoon_bluest_android_ScanCallbackProxy_nativeFinalize(
    _env: Env<'_>,
    _this: Arg<ScanCallback>,
    arc: RawArc<Box<dyn ScanCallbackProxy>>,
) {
    arc.destroy_arc();
}

pub trait BluetoothGattCallbackProxy: Send + Sync + 'static {
    fn onCharacteristicChanged_BluetoothGatt_BluetoothGattCharacteristic_byte_array<'env>(
        &self,
        env: Env<'env>,
        this: Ref<'env, BluetoothGattCallback>,
        gatt: Option<Ref<'env, BluetoothGatt>>,
        characteristic: Option<Ref<'env, BluetoothGattCharacteristic>>,
        value: Option<Ref<'env, ByteArray>>,
    ) {
    }

    fn onCharacteristicChanged_BluetoothGatt_BluetoothGattCharacteristic<'env>(
        &self,
        env: Env<'env>,
        this: Ref<'env, BluetoothGattCallback>,
        gatt: Option<Ref<'env, BluetoothGatt>>,
        characteristic: Option<Ref<'env, BluetoothGattCharacteristic>>,
    ) {
    }

    fn onCharacteristicRead_BluetoothGatt_BluetoothGattCharacteristic_byte_array_int<'env>(
        &self,
        env: Env<'env>,
        this: Ref<'env, BluetoothGattCallback>,
        gatt: Option<Ref<'env, BluetoothGatt>>,
        characteristic: Option<Ref<'env, BluetoothGattCharacteristic>>,
        value: Option<Ref<'env, ByteArray>>,
        status: i32,
    ) {
    }

    fn onCharacteristicRead_BluetoothGatt_BluetoothGattCharacteristic_int<'env>(
        &self,
        env: Env<'env>,
        this: Ref<'env, BluetoothGattCallback>,
        gatt: Option<Ref<'env, BluetoothGatt>>,
        characteristic: Option<Ref<'env, BluetoothGattCharacteristic>>,
        status: i32,
    ) {
    }

    fn onCharacteristicWrite<'env>(
        &self,
        env: Env<'env>,
        this: Ref<'env, BluetoothGattCallback>,
        gatt: Option<Ref<'env, BluetoothGatt>>,
        characteristic: Option<Ref<'env, BluetoothGattCharacteristic>>,
        status: i32,
    ) {
    }

    fn onConnectionStateChange<'env>(
        &self,
        env: Env<'env>,
        this: Ref<'env, BluetoothGattCallback>,
        gatt: Option<Ref<'env, BluetoothGatt>>,
        status: i32,
        new_state: i32,
    ) {
    }

    fn onDescriptorRead_BluetoothGatt_BluetoothGattDescriptor_int_byte_array<'env>(
        &self,
        env: Env<'env>,
        this: Ref<'env, BluetoothGattCallback>,
        gatt: Option<Ref<'env, BluetoothGatt>>,
        descriptor: Option<Ref<'env, BluetoothGattDescriptor>>,
        status: i32,
        value: Option<Ref<'env, ByteArray>>,
    ) {
    }

    fn onDescriptorRead_BluetoothGatt_BluetoothGattDescriptor_int<'env>(
        &self,
        env: Env<'env>,
        this: Ref<'env, BluetoothGattCallback>,
        gatt: Option<Ref<'env, BluetoothGatt>>,
        descriptor: Option<Ref<'env, BluetoothGattDescriptor>>,
        status: i32,
    ) {
    }

    fn onDescriptorWrite<'env>(
        &self,
        env: Env<'env>,
        this: Ref<'env, BluetoothGattCallback>,
        gatt: Option<Ref<'env, BluetoothGatt>>,
        descriptor: Option<Ref<'env, BluetoothGattDescriptor>>,
        status: i32,
    ) {
    }

    fn onMtuChanged<'env>(
        &self,
        env: Env<'env>,
        this: Ref<'env, BluetoothGattCallback>,
        gatt: Option<Ref<'env, BluetoothGatt>>,
        mtu: i32,
        status: i32,
    ) {
    }

    fn onPhyRead<'env>(
        &self,
        env: Env<'env>,
        this: Ref<'env, BluetoothGattCallback>,
        gatt: Option<Ref<'env, BluetoothGatt>>,
        tx_phy: i32,
        rx_phy: i32,
        status: i32,
    ) {
    }

    fn onPhyUpdate<'env>(
        &self,
        env: Env<'env>,
        this: Ref<'env, BluetoothGattCallback>,
        gatt: Option<Ref<'env, BluetoothGatt>>,
        tx_phy: i32,
        rx_phy: i32,
        status: i32,
    ) {
    }

    fn onReadRemoteRssi<'env>(
        &self,
        env: Env<'env>,
        this: Ref<'env, BluetoothGattCallback>,
        gatt: Option<Ref<'env, BluetoothGatt>>,
        rssi: i32,
        status: i32,
    ) {
    }

    fn onReliableWriteCompleted<'env>(
        &self,
        env: Env<'env>,
        this: Ref<'env, BluetoothGattCallback>,
        gatt: Option<Ref<'env, BluetoothGatt>>,
        status: i32,
    ) {
    }

    fn onServiceChanged<'env>(
        &self,
        env: Env<'env>,
        this: Ref<'env, BluetoothGattCallback>,
        gatt: Option<Ref<'env, BluetoothGatt>>,
    ) {
    }

    fn onServicesDiscovered<'env>(
        &self,
        env: Env<'env>,
        this: Ref<'env, BluetoothGattCallback>,
        gatt: Option<Ref<'env, BluetoothGatt>>,
        status: i32,
    ) {
    }
}

pub trait BluetoothGattCallbackProxyBuild {
    fn new_rust_proxy<'env, P: BluetoothGattCallbackProxy>(
        env: Env<'env>,
        proxy: P,
    ) -> Result<Local<'env, BluetoothGattCallback>, Local<'env, Throwable>>;
    fn rust_proxy_class(env: Env<'_>) -> &'static Global<Class>;
}

impl BluetoothGattCallbackProxyBuild for BluetoothGattCallback {
    fn new_rust_proxy<'env, P: BluetoothGattCallbackProxy>(
        env: Env<'env>,
        proxy: P,
    ) -> Result<Local<'env, BluetoothGattCallback>, Local<'env, Throwable>> {
        let arc = Arc::new(Box::new(proxy) as Box<dyn BluetoothGattCallbackProxy>);
        let proxy_jclass = Self::rust_proxy_class(env).as_raw();
        unsafe {
            let jni_args = [RawArc::from_arc(arc).into_jlong_value()];
            let jni_method = env.require_method(proxy_jclass, "<init>\0", "(J)V\0");
            env.new_object_a(proxy_jclass, jni_method, jni_args.as_ptr())
        }
    }

    fn rust_proxy_class(env: Env<'_>) -> &'static Global<Class> {
        static CLASS: OnceLock<Global<Class>> = OnceLock::new();
        CLASS.get_or_init(|| {
            let class_loader = DEX_CLASS_LOADER.as_ref(env);
            let class_jstring =
                JString::from_env_str(env, "com.github.alexmoon.bluest.android.BluetoothGattCallbackProxy");
            let class_object = class_loader.loadClass(class_jstring).unwrap().unwrap().as_global();

            unsafe {
                let (mut name_char_changed_new, mut sig_char_changed_new) = (*b"nativeOnCharacteristicChanged\0", *b"(JLandroid/bluetooth/BluetoothGatt;Landroid/bluetooth/BluetoothGattCharacteristic;[B)V\0");
                let (mut name_char_changed_old, mut sig_char_changed_old) = (*b"nativeOnCharacteristicChanged\0", *b"(JLandroid/bluetooth/BluetoothGatt;Landroid/bluetooth/BluetoothGattCharacteristic;)V\0");
                let (mut name_char_read_new, mut sig_char_read_new) = (*b"nativeOnCharacteristicRead\0", *b"(JLandroid/bluetooth/BluetoothGatt;Landroid/bluetooth/BluetoothGattCharacteristic;[BI)V\0");
                let (mut name_char_read_old, mut sig_char_read_old) = (*b"nativeOnCharacteristicRead\0", *b"(JLandroid/bluetooth/BluetoothGatt;Landroid/bluetooth/BluetoothGattCharacteristic;I)V\0");
                let (mut name_char_write, mut sig_char_write) = (*b"nativeOnCharacteristicWrite\0", *b"(JLandroid/bluetooth/BluetoothGatt;Landroid/bluetooth/BluetoothGattCharacteristic;I)V\0");
                let (mut name_conn_state_change, mut sig_conn_state_change) = (*b"nativeOnConnectionStateChange\0", *b"(JLandroid/bluetooth/BluetoothGatt;II)V\0");
                let (mut name_desc_read_new, mut sig_desc_read_new) = (*b"nativeOnDescriptorRead\0", *b"(JLandroid/bluetooth/BluetoothGatt;Landroid/bluetooth/BluetoothGattDescriptor;I[B)V\0");
                let (mut name_desc_read_old, mut sig_desc_read_old) = (*b"nativeOnDescriptorRead\0", *b"(JLandroid/bluetooth/BluetoothGatt;Landroid/bluetooth/BluetoothGattDescriptor;I)V\0");
                let (mut name_desc_write, mut sig_desc_write) = (*b"nativeOnDescriptorWrite\0", *b"(JLandroid/bluetooth/BluetoothGatt;Landroid/bluetooth/BluetoothGattDescriptor;I)V\0");
                let (mut name_mtu_changed, mut sig_mtu_changed) = (*b"nativeOnMtuChanged\0", *b"(JLandroid/bluetooth/BluetoothGatt;II)V\0");
                let (mut name_phy_read, mut sig_phy_read) = (*b"nativeOnPhyRead\0", *b"(JLandroid/bluetooth/BluetoothGatt;III)V\0");
                let (mut name_phy_update, mut sig_phy_update) = (*b"nativeOnPhyUpdate\0", *b"(JLandroid/bluetooth/BluetoothGatt;III)V\0");
                let (mut name_read_remote_rssi, mut sig_read_remote_rssi) = (*b"nativeOnReadRemoteRssi\0", *b"(JLandroid/bluetooth/BluetoothGatt;II)V\0");
                let (mut name_reliable_write_comp, mut sig_reliable_write_comp) = (*b"nativeOnReliableWriteCompleted\0", *b"(JLandroid/bluetooth/BluetoothGatt;I)V\0");
                let (mut name_service_changed, mut sig_service_changed) = (*b"nativeOnServiceChanged\0", *b"(JLandroid/bluetooth/BluetoothGatt;)V\0");
                let (mut name_service_discover, mut sig_service_discover) = (*b"nativeOnServicesDiscovered\0", *b"(JLandroid/bluetooth/BluetoothGatt;I)V\0");
                let (mut name_finalize, mut sig_finalize) = (*b"nativeFinalize\0", *b"(J)V\0");

                let mut native_methods = vec![
                    java_spaghetti::sys::JNINativeMethod {
                        name: name_char_changed_old.as_mut_ptr(),
                        signature: sig_char_changed_old.as_mut_ptr(),
                        fnPtr: Java_com_github_alexmoon_bluest_android_BluetoothGattCallbackProxy_onCharacteristicChanged_BluetoothGatt_BluetoothGattCharacteristic as *mut _,
                    },
                    java_spaghetti::sys::JNINativeMethod {
                        name: name_char_read_old.as_mut_ptr(),
                        signature: sig_char_read_old.as_mut_ptr(),
                        fnPtr: Java_com_github_alexmoon_bluest_android_BluetoothGattCallbackProxy_onCharacteristicRead_BluetoothGatt_BluetoothGattCharacteristic_int as *mut _,
                    },
                    java_spaghetti::sys::JNINativeMethod {
                        name: name_char_write.as_mut_ptr(),
                        signature: sig_char_write.as_mut_ptr(),
                        fnPtr: Java_com_github_alexmoon_bluest_android_BluetoothGattCallbackProxy_onCharacteristicWrite as *mut _,
                    },
                    java_spaghetti::sys::JNINativeMethod {
                        name: name_conn_state_change.as_mut_ptr(),
                        signature: sig_conn_state_change.as_mut_ptr(),
                        fnPtr: Java_com_github_alexmoon_bluest_android_BluetoothGattCallbackProxy_onConnectionStateChange as *mut _,
                    },

                    java_spaghetti::sys::JNINativeMethod {
                        name: name_desc_read_old.as_mut_ptr(),
                        signature: sig_desc_read_old.as_mut_ptr(),
                        fnPtr: Java_com_github_alexmoon_bluest_android_BluetoothGattCallbackProxy_onDescriptorRead_BluetoothGatt_BluetoothGattDescriptor_int as *mut _,
                    },
                    java_spaghetti::sys::JNINativeMethod {
                        name: name_desc_write.as_mut_ptr(),
                        signature: sig_desc_write.as_mut_ptr(),
                        fnPtr: Java_com_github_alexmoon_bluest_android_BluetoothGattCallbackProxy_onDescriptorWrite as *mut _,
                    },
                    java_spaghetti::sys::JNINativeMethod {
                        name: name_read_remote_rssi.as_mut_ptr(),
                        signature: sig_read_remote_rssi.as_mut_ptr(),
                        fnPtr: Java_com_github_alexmoon_bluest_android_BluetoothGattCallbackProxy_onReadRemoteRssi as *mut _,
                    },
                    java_spaghetti::sys::JNINativeMethod {
                        name: name_reliable_write_comp.as_mut_ptr(),
                        signature: sig_reliable_write_comp.as_mut_ptr(),
                        fnPtr: Java_com_github_alexmoon_bluest_android_BluetoothGattCallbackProxy_onReliableWriteCompleted as *mut _,
                    },
                    java_spaghetti::sys::JNINativeMethod {
                        name: name_service_discover.as_mut_ptr(),
                        signature: sig_service_discover.as_mut_ptr(),
                        fnPtr: Java_com_github_alexmoon_bluest_android_BluetoothGattCallbackProxy_onServicesDiscovered as *mut _,
                    },
                    java_spaghetti::sys::JNINativeMethod {
                        name: name_finalize.as_mut_ptr(),
                        signature: sig_finalize.as_mut_ptr(),
                        fnPtr: Java_com_github_alexmoon_bluest_android_BluetoothGattCallbackProxy_nativeFinalize as *mut _,
                    },
                ];

                let android_api_level = Build_VERSION::SDK_INT(env);

                if android_api_level >= 21 {
                    native_methods.extend_from_slice(&[
                        java_spaghetti::sys::JNINativeMethod {
                            name: name_mtu_changed.as_mut_ptr(),
                            signature: sig_mtu_changed.as_mut_ptr(),
                            fnPtr: Java_com_github_alexmoon_bluest_android_BluetoothGattCallbackProxy_onMtuChanged as *mut _,
                        }
                    ]);
                }

                if android_api_level >= 26 {
                    native_methods.extend_from_slice(&[
                        java_spaghetti::sys::JNINativeMethod {
                            name: name_phy_read.as_mut_ptr(),
                            signature: sig_phy_read.as_mut_ptr(),
                            fnPtr: Java_com_github_alexmoon_bluest_android_BluetoothGattCallbackProxy_onPhyRead as *mut _,
                        },
                        java_spaghetti::sys::JNINativeMethod {
                            name: name_phy_update.as_mut_ptr(),
                            signature: sig_phy_update.as_mut_ptr(),
                            fnPtr: Java_com_github_alexmoon_bluest_android_BluetoothGattCallbackProxy_onPhyUpdate as *mut _,
                        },
                    ]);
                }

                if android_api_level >= 31 {
                    native_methods.extend_from_slice(&[
                        java_spaghetti::sys::JNINativeMethod {
                            name: name_service_changed.as_mut_ptr(),
                            signature: sig_service_changed.as_mut_ptr(),
                            fnPtr: Java_com_github_alexmoon_bluest_android_BluetoothGattCallbackProxy_onServiceChanged as *mut _,
                        },
                    ]);
                }

                if android_api_level >= 33 {
                    native_methods.extend_from_slice(&[
                        java_spaghetti::sys::JNINativeMethod {
                            name: name_char_changed_new.as_mut_ptr(),
                            signature: sig_char_changed_new.as_mut_ptr(),
                            fnPtr: Java_com_github_alexmoon_bluest_android_BluetoothGattCallbackProxy_onCharacteristicChanged_BluetoothGatt_BluetoothGattCharacteristic_byte_array as *mut _,
                        },
                        java_spaghetti::sys::JNINativeMethod {
                            name: name_char_read_new.as_mut_ptr(),
                            signature: sig_char_read_new.as_mut_ptr(),
                            fnPtr: Java_com_github_alexmoon_bluest_android_BluetoothGattCallbackProxy_onCharacteristicRead_BluetoothGatt_BluetoothGattCharacteristic_byte_array_int as *mut _,
                        },
                        java_spaghetti::sys::JNINativeMethod {
                            name: name_desc_read_new.as_mut_ptr(),
                            signature: sig_desc_read_new.as_mut_ptr(),
                            fnPtr: Java_com_github_alexmoon_bluest_android_BluetoothGattCallbackProxy_onDescriptorRead_BluetoothGatt_BluetoothGattDescriptor_int_byte_array as *mut _,
                        },
                    ]);
                }

                ((**env.as_raw()).v1_2.RegisterNatives)(
                    env.as_raw(),
                    class_object.as_raw(),
                    native_methods.as_mut_ptr(),
                    native_methods.len() as i32,
                );
            }

            class_object
        })
    }
}

unsafe extern "system" fn Java_com_github_alexmoon_bluest_android_BluetoothGattCallbackProxy_onCharacteristicChanged_BluetoothGatt_BluetoothGattCharacteristic_byte_array<
    'env,
>(
    env: Env<'env>,
    this: Arg<BluetoothGattCallback>,
    arc: RawArc<Box<dyn BluetoothGattCallbackProxy>>,
    gatt: Arg<BluetoothGatt>,
    characteristic: Arg<BluetoothGattCharacteristic>,
    value: Arg<ByteArray>,
) {
    let arc = arc.clone_arc();
    arc.onCharacteristicChanged_BluetoothGatt_BluetoothGattCharacteristic_byte_array(
        env,
        this.into_ref(env).unwrap(),
        gatt.into_ref(env),
        characteristic.into_ref(env),
        value.into_ref(env),
    );
}

unsafe extern "system" fn Java_com_github_alexmoon_bluest_android_BluetoothGattCallbackProxy_onCharacteristicChanged_BluetoothGatt_BluetoothGattCharacteristic<
    'env,
>(
    env: Env<'env>,
    this: Arg<BluetoothGattCallback>,
    arc: RawArc<Box<dyn BluetoothGattCallbackProxy>>,
    gatt: Arg<BluetoothGatt>,
    characteristic: Arg<BluetoothGattCharacteristic>,
) {
    let arc = arc.clone_arc();
    arc.onCharacteristicChanged_BluetoothGatt_BluetoothGattCharacteristic(
        env,
        this.into_ref(env).unwrap(),
        gatt.into_ref(env),
        characteristic.into_ref(env),
    );
}

unsafe extern "system" fn Java_com_github_alexmoon_bluest_android_BluetoothGattCallbackProxy_onCharacteristicRead_BluetoothGatt_BluetoothGattCharacteristic_byte_array_int<
    'env,
>(
    env: Env<'env>,
    this: Arg<BluetoothGattCallback>,
    arc: RawArc<Box<dyn BluetoothGattCallbackProxy>>,
    gatt: Arg<BluetoothGatt>,
    characteristic: Arg<BluetoothGattCharacteristic>,
    value: Arg<ByteArray>,
    status: i32,
) {
    let arc = arc.clone_arc();
    arc.onCharacteristicRead_BluetoothGatt_BluetoothGattCharacteristic_byte_array_int(
        env,
        this.into_ref(env).unwrap(),
        gatt.into_ref(env),
        characteristic.into_ref(env),
        value.into_ref(env),
        status,
    );
}

unsafe extern "system" fn Java_com_github_alexmoon_bluest_android_BluetoothGattCallbackProxy_onCharacteristicRead_BluetoothGatt_BluetoothGattCharacteristic_int<
    'env,
>(
    env: Env<'env>,
    this: Arg<BluetoothGattCallback>,
    arc: RawArc<Box<dyn BluetoothGattCallbackProxy>>,
    gatt: Arg<BluetoothGatt>,
    characteristic: Arg<BluetoothGattCharacteristic>,
    status: i32,
) {
    let arc = arc.clone_arc();
    arc.onCharacteristicRead_BluetoothGatt_BluetoothGattCharacteristic_int(
        env,
        this.into_ref(env).unwrap(),
        gatt.into_ref(env),
        characteristic.into_ref(env),
        status,
    );
}

unsafe extern "system" fn Java_com_github_alexmoon_bluest_android_BluetoothGattCallbackProxy_onCharacteristicWrite<
    'env,
>(
    env: Env<'env>,
    this: Arg<BluetoothGattCallback>,
    arc: RawArc<Box<dyn BluetoothGattCallbackProxy>>,
    gatt: Arg<BluetoothGatt>,
    characteristic: Arg<BluetoothGattCharacteristic>,
    status: i32,
) {
    let arc = arc.clone_arc();
    arc.onCharacteristicWrite(
        env,
        this.into_ref(env).unwrap(),
        gatt.into_ref(env),
        characteristic.into_ref(env),
        status,
    );
}

unsafe extern "system" fn Java_com_github_alexmoon_bluest_android_BluetoothGattCallbackProxy_onConnectionStateChange<
    'env,
>(
    env: Env<'env>,
    this: Arg<BluetoothGattCallback>,
    arc: RawArc<Box<dyn BluetoothGattCallbackProxy>>,
    gatt: Arg<BluetoothGatt>,
    status: i32,
    new_state: i32,
) {
    let arc = arc.clone_arc();
    arc.onConnectionStateChange(env, this.into_ref(env).unwrap(), gatt.into_ref(env), status, new_state);
}

unsafe extern "system" fn Java_com_github_alexmoon_bluest_android_BluetoothGattCallbackProxy_onDescriptorRead_BluetoothGatt_BluetoothGattDescriptor_int_byte_array<
    'env,
>(
    env: Env<'env>,
    this: Arg<BluetoothGattCallback>,
    arc: RawArc<Box<dyn BluetoothGattCallbackProxy>>,
    gatt: Arg<BluetoothGatt>,
    descriptor: Arg<BluetoothGattDescriptor>,
    status: i32,
    value: Arg<ByteArray>,
) {
    let arc = arc.clone_arc();
    arc.onDescriptorRead_BluetoothGatt_BluetoothGattDescriptor_int_byte_array(
        env,
        this.into_ref(env).unwrap(),
        gatt.into_ref(env),
        descriptor.into_ref(env),
        status,
        value.into_ref(env),
    );
}

unsafe extern "system" fn Java_com_github_alexmoon_bluest_android_BluetoothGattCallbackProxy_onDescriptorRead_BluetoothGatt_BluetoothGattDescriptor_int<
    'env,
>(
    env: Env<'env>,
    this: Arg<BluetoothGattCallback>,
    arc: RawArc<Box<dyn BluetoothGattCallbackProxy>>,
    gatt: Arg<BluetoothGatt>,
    descriptor: Arg<BluetoothGattDescriptor>,
    status: i32,
) {
    let arc = arc.clone_arc();
    arc.onDescriptorRead_BluetoothGatt_BluetoothGattDescriptor_int(
        env,
        this.into_ref(env).unwrap(),
        gatt.into_ref(env),
        descriptor.into_ref(env),
        status,
    );
}

unsafe extern "system" fn Java_com_github_alexmoon_bluest_android_BluetoothGattCallbackProxy_onDescriptorWrite<'env>(
    env: Env<'env>,
    this: Arg<BluetoothGattCallback>,
    arc: RawArc<Box<dyn BluetoothGattCallbackProxy>>,
    gatt: Arg<BluetoothGatt>,
    descriptor: Arg<BluetoothGattDescriptor>,
    status: i32,
) {
    let arc = arc.clone_arc();
    arc.onDescriptorWrite(
        env,
        this.into_ref(env).unwrap(),
        gatt.into_ref(env),
        descriptor.into_ref(env),
        status,
    );
}

unsafe extern "system" fn Java_com_github_alexmoon_bluest_android_BluetoothGattCallbackProxy_onMtuChanged<'env>(
    env: Env<'env>,
    this: Arg<BluetoothGattCallback>,
    arc: RawArc<Box<dyn BluetoothGattCallbackProxy>>,
    gatt: Arg<BluetoothGatt>,
    mtu: i32,
    status: i32,
) {
    let arc = arc.clone_arc();
    arc.onMtuChanged(env, this.into_ref(env).unwrap(), gatt.into_ref(env), mtu, status);
}

unsafe extern "system" fn Java_com_github_alexmoon_bluest_android_BluetoothGattCallbackProxy_onPhyRead<'env>(
    env: Env<'env>,
    this: Arg<BluetoothGattCallback>,
    arc: RawArc<Box<dyn BluetoothGattCallbackProxy>>,
    gatt: Arg<BluetoothGatt>,
    tx_phy: i32,
    rx_phy: i32,
    status: i32,
) {
    let arc = arc.clone_arc();
    arc.onPhyRead(
        env,
        this.into_ref(env).unwrap(),
        gatt.into_ref(env),
        tx_phy,
        rx_phy,
        status,
    );
}

unsafe extern "system" fn Java_com_github_alexmoon_bluest_android_BluetoothGattCallbackProxy_onPhyUpdate<'env>(
    env: Env<'env>,
    this: Arg<BluetoothGattCallback>,
    arc: RawArc<Box<dyn BluetoothGattCallbackProxy>>,
    gatt: Arg<BluetoothGatt>,
    tx_phy: i32,
    rx_phy: i32,
    status: i32,
) {
    let arc = arc.clone_arc();
    arc.onPhyUpdate(
        env,
        this.into_ref(env).unwrap(),
        gatt.into_ref(env),
        tx_phy,
        rx_phy,
        status,
    );
}

unsafe extern "system" fn Java_com_github_alexmoon_bluest_android_BluetoothGattCallbackProxy_onReadRemoteRssi<'env>(
    env: Env<'env>,
    this: Arg<BluetoothGattCallback>,
    arc: RawArc<Box<dyn BluetoothGattCallbackProxy>>,
    gatt: Arg<BluetoothGatt>,
    rssi: i32,
    status: i32,
) {
    let arc = arc.clone_arc();
    arc.onReadRemoteRssi(env, this.into_ref(env).unwrap(), gatt.into_ref(env), rssi, status);
}

unsafe extern "system" fn Java_com_github_alexmoon_bluest_android_BluetoothGattCallbackProxy_onReliableWriteCompleted<
    'env,
>(
    env: Env<'env>,
    this: Arg<BluetoothGattCallback>,
    arc: RawArc<Box<dyn BluetoothGattCallbackProxy>>,
    gatt: Arg<BluetoothGatt>,
    status: i32,
) {
    let arc = arc.clone_arc();
    arc.onReliableWriteCompleted(env, this.into_ref(env).unwrap(), gatt.into_ref(env), status);
}

unsafe extern "system" fn Java_com_github_alexmoon_bluest_android_BluetoothGattCallbackProxy_onServiceChanged<'env>(
    env: Env<'env>,
    this: Arg<BluetoothGattCallback>,
    arc: RawArc<Box<dyn BluetoothGattCallbackProxy>>,
    gatt: Arg<BluetoothGatt>,
) {
    let arc = arc.clone_arc();
    arc.onServiceChanged(env, this.into_ref(env).unwrap(), gatt.into_ref(env));
}

unsafe extern "system" fn Java_com_github_alexmoon_bluest_android_BluetoothGattCallbackProxy_onServicesDiscovered<
    'env,
>(
    env: Env<'env>,
    this: Arg<BluetoothGattCallback>,
    arc: RawArc<Box<dyn BluetoothGattCallbackProxy>>,
    gatt: Arg<BluetoothGatt>,
    status: i32,
) {
    let arc = arc.clone_arc();
    arc.onServicesDiscovered(env, this.into_ref(env).unwrap(), gatt.into_ref(env), status);
}

unsafe extern "system" fn Java_com_github_alexmoon_bluest_android_BluetoothGattCallbackProxy_nativeFinalize(
    _env: Env<'_>,
    _this: Arg<ScanCallback>,
    arc: RawArc<Box<dyn BluetoothGattCallbackProxy>>,
) {
    arc.destroy_arc();
}
