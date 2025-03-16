use std::collections::HashMap;
use std::sync::Arc;

use async_channel::{Receiver, Sender};
use futures_core::Stream;
use futures_lite::{stream, StreamExt};
use java_spaghetti::{ByteArray, Env, Global, Local, Null, PrimitiveArray, Ref};
use tracing::{debug, warn};
use uuid::Uuid;

use super::bindings::android::bluetooth::le::{
    BluetoothLeScanner, ScanCallback, ScanResult, ScanSettings, ScanSettings_Builder,
};
use super::bindings::android::bluetooth::{BluetoothAdapter, BluetoothManager};
use super::bindings::android::os::{Build_VERSION, ParcelUuid};
use super::bindings::java::lang::{String as JString, Throwable};
use super::bindings::java::util::Map_Entry;
use super::callbacks::{ScanCallbackProxy, ScanCallbackProxyBuild};
use super::device::DeviceImpl;
use super::{vm_context, JavaIterator, OptionExt};

use crate::util::defer;
use crate::{
    AdapterEvent, AdvertisementData, AdvertisingDevice, ConnectionEvent, Device, DeviceId, ManufacturerData, Result,
};

struct AdapterInner {
    manager: Global<BluetoothManager>,
    _adapter: Global<BluetoothAdapter>,
    le_scanner: Global<BluetoothLeScanner>,
}

#[derive(Clone)]
pub struct AdapterImpl {
    inner: Arc<AdapterInner>,
}

impl AdapterImpl {
    pub async fn default() -> Option<Self> {
        if !vm_context::ndk_context_available() {
            return None;
        };
        use super::bindings::android::content::Context;
        let context = vm_context::get_android_context();

        vm_context::get_vm().with_env(|env| {
            let local_context = context.as_ref(env);
            let service_name = JString::from_env_str(env, Context::BLUETOOTH_SERVICE);
            let manager = local_context
                .getSystemService_String(service_name)
                .ok()??
                .cast::<BluetoothManager>()
                .ok()?;
            let local_manager = manager.as_ref();
            let adapter = local_manager.getAdapter().ok()??;
            let le_scanner = adapter.getBluetoothLeScanner().ok()??;

            Some(Self {
                inner: Arc::new(AdapterInner {
                    _adapter: adapter.as_global(),
                    le_scanner: le_scanner.as_global(),
                    manager: manager.as_global(),
                }),
            })
        })
    }

    pub(crate) async fn events(&self) -> Result<impl Stream<Item = Result<AdapterEvent>> + Send + Unpin + '_> {
        Ok(stream::empty()) // TODO
    }

    pub async fn wait_available(&self) -> Result<()> {
        Ok(())
    }

    pub async fn open_device(&self, _id: &DeviceId) -> Result<Device> {
        todo!()
    }

    pub async fn connected_devices(&self) -> Result<Vec<Device>> {
        todo!()
    }

    pub async fn connected_devices_with_services(&self, _services: &[Uuid]) -> Result<Vec<Device>> {
        todo!()
    }

    pub async fn scan<'a>(
        &'a self,
        _services: &'a [Uuid],
    ) -> Result<impl Stream<Item = AdvertisingDevice> + Send + Unpin + 'a> {
        self.inner.manager.vm().with_env(|env| {
            let (callback, receiver) = BluestScanCallback::build(env)?;
            let callback_global = callback.as_global();
            let scanner = self.inner.le_scanner.as_ref(env);
            let settings = ScanSettings_Builder::new(env)?;
            settings.setScanMode(ScanSettings::SCAN_MODE_LOW_LATENCY)?;
            let settings = settings.build()?.non_null()?;
            scanner.startScan_List_ScanSettings_ScanCallback(Null, settings, callback)?;

            let guard = defer(move || {
                self.inner.manager.vm().with_env(|env| {
                    let callback = callback_global.as_ref(env);
                    let scanner = self.inner.le_scanner.as_ref(env);
                    match scanner.stopScan_ScanCallback(callback) {
                        Ok(()) => debug!("stopped scan"),
                        Err(e) => warn!("failed to stop scan: {:?}", e),
                    };
                });
            });

            Ok(Box::pin(receiver).map(move |x| {
                let _guard = &guard;
                x
            }))
        })
    }

    pub async fn discover_devices<'a>(
        &'a self,
        services: &'a [Uuid],
    ) -> Result<impl Stream<Item = Result<Device>> + Send + Unpin + 'a> {
        let connected = stream::iter(self.connected_devices_with_services(services).await?).map(Ok);

        // try_unfold is used to ensure we do not start scanning until the connected devices have been consumed
        let advertising = Box::pin(stream::try_unfold(None, |state| async {
            let mut stream = match state {
                Some(stream) => stream,
                None => self.scan(services).await?,
            };
            Ok(stream.next().await.map(|x| (x.device, Some(stream))))
        }));

        Ok(connected.chain(advertising))
    }

    pub async fn connect_device(&self, _device: &Device) -> Result<()> {
        // Windows manages the device connection automatically
        Ok(())
    }

    pub async fn disconnect_device(&self, _device: &Device) -> Result<()> {
        // Windows manages the device connection automatically
        Ok(())
    }

    pub async fn device_connection_events<'a>(
        &'a self,
        _device: &'a Device,
    ) -> Result<impl Stream<Item = ConnectionEvent> + Send + Unpin + 'a> {
        Ok(stream::empty()) // TODO
    }
}

impl PartialEq for AdapterImpl {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Eq for AdapterImpl {}

impl std::hash::Hash for AdapterImpl {
    fn hash<H: std::hash::Hasher>(&self, _state: &mut H) {}
}

impl std::fmt::Debug for AdapterImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Adapter").finish()
    }
}

fn convert_uuid(uuid: Local<'_, ParcelUuid>) -> Result<Uuid> {
    let uuid = uuid.getUuid()?.non_null()?;
    let lsb = uuid.getLeastSignificantBits()? as u64;
    let msb = uuid.getMostSignificantBits()? as u64;
    Ok(Uuid::from_u64_pair(msb, lsb))
}

fn on_scan_result(env: Env<'_>, callback_type: i32, scan_result: Ref<ScanResult>) -> Result<AdvertisingDevice> {
    tracing::info!("got callback! {}", callback_type);

    let scan_record = scan_result.getScanRecord()?.non_null()?;
    let device = scan_result.getDevice()?.non_null()?;

    let address = device.getAddress()?.non_null()?.to_string_lossy();
    let rssi = scan_result.getRssi()?;
    let is_connectable = if Build_VERSION::SDK_INT(env) >= 26 {
        scan_result.isConnectable()?
    } else {
        // TODO confirm that it is really unavailable, and make `is_connectable` an `Option`;
        // or try to check `eventType` via `toString()` (see `ScanResult.java`).
        true
    };
    let local_name = scan_record.getDeviceName()?.map(|s| s.to_string_lossy());
    let tx_power_level = scan_record.getTxPowerLevel()?;

    // Services
    let mut services = Vec::new();
    if let Some(uuids) = scan_record.getServiceUuids()? {
        for uuid in JavaIterator(uuids.iterator()?.non_null()?) {
            services.push(convert_uuid(uuid.cast()?)?)
        }
    }

    // Service data
    let mut service_data = HashMap::new();
    let sd = scan_record.getServiceData()?.non_null()?;
    let sd = sd.entrySet()?.non_null()?;
    for entry in JavaIterator(sd.iterator()?.non_null()?) {
        let entry: Local<Map_Entry> = entry.cast()?;
        let key: Local<ParcelUuid> = entry.getKey()?.non_null()?.cast()?;
        let val: Local<ByteArray> = entry.getValue()?.non_null()?.cast()?;
        service_data.insert(convert_uuid(key)?, val.as_vec().into_iter().map(|i| i as u8).collect());
    }

    // Manufacturer data
    let mut manufacturer_data = None;
    let msd = scan_record.getManufacturerSpecificData()?.non_null()?;
    // TODO there can be multiple manufacturer data entries, but the bluest API only supports one. So grab just the first.
    if msd.size()? != 0 {
        let val: Local<'_, ByteArray> = msd.valueAt(0)?.non_null()?.cast()?;
        manufacturer_data = Some(ManufacturerData {
            company_id: msd.keyAt(0)? as _,
            data: val.as_vec().into_iter().map(|i| i as u8).collect(),
        });
    }

    let device_id = DeviceId(address);

    Ok(AdvertisingDevice {
        device: Device(DeviceImpl {
            id: device_id,
            device: device.as_global(),
        }),
        adv_data: AdvertisementData {
            is_connectable,
            local_name,
            manufacturer_data, // TODO, SparseArray is cursed.
            service_data,
            services,
            tx_power_level: Some(tx_power_level as _),
        },
        rssi: Some(rssi as _),
    })
}

struct BluestScanCallback {
    sender: Sender<AdvertisingDevice>,
}

impl BluestScanCallback {
    fn build(env: Env<'_>) -> Result<(Local<'_, ScanCallback>, Receiver<AdvertisingDevice>), Local<'_, Throwable>> {
        let (sender, receiver) = async_channel::bounded(16);
        let proxy = Arc::new(Box::new(Self { sender }) as Box<dyn ScanCallbackProxy>);
        Ok((ScanCallback::new_rust_proxy(env, proxy)?, receiver))
    }
}

impl ScanCallbackProxy for BluestScanCallback {
    fn onScanResult<'env>(
        &self,
        env: Env<'env>,
        _this: java_spaghetti::Ref<ScanCallback>,
        callback_type: i32,
        result: Option<java_spaghetti::Ref<'env, ScanResult>>,
    ) {
        let Some(result) = result else {
            return;
        };
        let adv_dev = match on_scan_result(env, callback_type, result) {
            Ok(dev) => dev,
            Err(e) => {
                warn!("failed to process the scan result: {:?}", e);
                return;
            }
        };
        if let Err(e) = self.sender.send_blocking(adv_dev) {
            warn!("failed to send scan callback: {:?}", e)
        }
    }

    fn onScanFailed<'env>(&self, _env: Env<'env>, _this: java_spaghetti::Ref<ScanCallback>, error_code: i32) {
        tracing::error!("got scan fail! {}", error_code);
    }
}
