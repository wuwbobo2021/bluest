pub use android_ble::DeviceId;
pub mod adapter;
pub mod characteristic;
pub mod descriptor;
pub mod device;
pub mod l2cap_channel;
pub mod service;

impl From<android_ble::Error> for crate::Error {
    fn from(err: android_ble::Error) -> Self {
        Self::new(err.kind().into(), err.source_cloned(), err.message())
    }
}

impl From<android_ble::error::ErrorKind> for crate::error::ErrorKind {
    fn from(value: android_ble::error::ErrorKind) -> Self {
        use crate::error::ErrorKind as DstErrKind;
        use android_ble::error::ErrorKind as SrcErrKind;
        match value {
            SrcErrKind::AdapterUnavailable => DstErrKind::AdapterUnavailable,
            SrcErrKind::AlreadyScanning => DstErrKind::AlreadyScanning,
            SrcErrKind::ConnectionFailed => DstErrKind::ConnectionFailed,
            SrcErrKind::NotConnected => DstErrKind::NotConnected,
            SrcErrKind::NotSupported => DstErrKind::NotSupported,
            SrcErrKind::NotAuthorized => DstErrKind::NotAuthorized,
            SrcErrKind::NotReady => DstErrKind::NotReady,
            SrcErrKind::NotFound => DstErrKind::NotFound,
            SrcErrKind::InvalidParameter => DstErrKind::InvalidParameter,
            SrcErrKind::Timeout => DstErrKind::Timeout,
            SrcErrKind::Protocol(att) => DstErrKind::Protocol(att.into()),
            SrcErrKind::Internal => DstErrKind::Internal,
            SrcErrKind::ServiceChanged => DstErrKind::ServiceChanged,
            SrcErrKind::Other => DstErrKind::Other,
            _ => DstErrKind::Other,
        }
    }
}

impl From<android_ble::error::AttError> for crate::error::AttError {
    fn from(value: android_ble::error::AttError) -> Self {
        Self::from_u8(value.as_u8())
    }
}
