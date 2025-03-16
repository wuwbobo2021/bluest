// TODO: remove this module after publishing a new version of `java-spaghetti`.

use std::cell::{Cell, OnceCell};
use std::ptr::null_mut;

use java_spaghetti::{sys::*, Env};

/// FFI: Use **&VM** instead of *const JavaVM.  This represents a global, process-wide Java exection environment.
///
/// On Android, there is only one VM per-process, although on desktop it's possible (if rare) to have multiple VMs
/// within the same process.  This library does not support having multiple VMs active simultaniously.
#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VM(*mut JavaVM);

impl VM {
    pub fn as_raw(&self) -> *mut JavaVM {
        self.0
    }

    /// Constructs `VM` with a *valid* non-null `jni_sys::JavaVM` raw pointer.
    ///
    /// # Safety
    ///
    /// - Make sure the corresponding JVM will keep alive within the lifetime of current native library or application.
    /// - Do not use any class redefinition feature, which may break the validity of method/field IDs to be cached.
    pub unsafe fn from_raw(vm: *mut JavaVM) -> Self {
        Self(vm)
    }

    pub fn with_env<F, R>(&self, callback: F) -> R
    where
        F: for<'env> FnOnce(Env<'env>) -> R,
    {
        let mut env = null_mut();
        let just_attached = match unsafe { ((**self.0).v1_2.GetEnv)(self.0, &mut env, JNI_VERSION_1_2) } {
            JNI_OK => false,
            JNI_EDETACHED => {
                let ret = unsafe { ((**self.0).v1_2.AttachCurrentThread)(self.0, &mut env, null_mut()) };
                if ret != JNI_OK {
                    panic!("AttachCurrentThread returned unknown error: {}", ret)
                }
                if !get_thread_exit_flag() {
                    set_thread_attach_flag(self.0);
                }
                true
            }
            JNI_EVERSION => panic!("GetEnv returned JNI_EVERSION"),
            unexpected => panic!("GetEnv returned unknown error: {}", unexpected),
        };

        let result = callback(unsafe { Env::from_raw(env as _) });

        if just_attached && get_thread_exit_flag() {
            // this is needed in case of `with_env` is used on dropping some thread-local instance.
            unsafe { ((**self.0).v1_2.DetachCurrentThread)(self.0) };
        }

        result
    }
}

unsafe impl Send for VM {}
unsafe impl Sync for VM {}

impl From<VM> for java_spaghetti::VM {
    fn from(vm: VM) -> Self {
        unsafe { java_spaghetti::VM::from_raw(vm.as_raw()) }
    }
}

thread_local! {
    static THREAD_ATTACH_FLAG: Cell<Option<AttachFlag>> = const { Cell::new(None) };
    static THREAD_EXIT_FLAG: OnceCell<()> = const { OnceCell::new() };
}

struct AttachFlag {
    raw_vm: *mut JavaVM,
}

impl Drop for AttachFlag {
    fn drop(&mut self) {
        // avoids the fatal error "Native thread exiting without having called DetachCurrentThread"
        unsafe { ((**self.raw_vm).v1_2.DetachCurrentThread)(self.raw_vm) };
        let _ = THREAD_EXIT_FLAG.try_with(|flag| flag.set(()));
    }
}

fn set_thread_attach_flag(raw_vm: *mut JavaVM) {
    THREAD_ATTACH_FLAG.replace(Some(AttachFlag { raw_vm }));
}

fn get_thread_exit_flag() -> bool {
    THREAD_EXIT_FLAG.try_with(|flag| flag.get().is_some()).unwrap_or(true)
}
