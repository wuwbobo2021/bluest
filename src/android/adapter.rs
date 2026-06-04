use futures_core::Stream;
use futures_lite::StreamExt;
use uuid::Uuid;

use super::device::DeviceImpl;
use super::DeviceId;
use crate::{
    AdapterEvent, AdvertisementData, AdvertisingDevice, ConnectionEvent, Device, Error, ManufacturerData, Result,
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AdapterImpl(android_ble::Adapter);

pub use android_ble::AdapterConfig;

impl AdapterImpl {
    /// Creates an interface to a Bluetooth adapter.
    pub async fn with_config(config: AdapterConfig) -> Result<Self> {
        let adapter = android_ble::Adapter::with_config(config).await?;
        Ok(AdapterImpl(adapter))
    }

    pub(crate) async fn events(&self) -> Result<impl Stream<Item = Result<AdapterEvent>> + Send + Unpin + '_> {
        Ok(self.0.events().await?.map(|e| {
            e.map(|e| match e {
                android_ble::AdapterEvent::Available => AdapterEvent::Available,
                android_ble::AdapterEvent::Unavailable => AdapterEvent::Unavailable,
            })
            .map_err(Error::from)
        }))
    }

    pub async fn wait_available(&self) -> Result<()> {
        self.0.wait_available().await.map_err(Error::from)
    }

    /// Check if the adapter is available
    pub async fn is_available(&self) -> Result<bool> {
        self.0.is_available().await.map_err(Error::from)
    }

    pub async fn open_device(&self, id: &DeviceId) -> Result<Device> {
        self.0
            .open_device(id)
            .await
            .map(|dev| Device(DeviceImpl(dev)))
            .map_err(Error::from)
    }

    pub async fn connected_devices(&self) -> Result<Vec<Device>> {
        self.0
            .connected_devices()
            .await
            .map(convert_devices)
            .map_err(Error::from)
    }

    pub async fn connected_devices_with_services(&self, services: &[Uuid]) -> Result<Vec<Device>> {
        self.0
            .connected_devices_with_services(services)
            .await
            .map(convert_devices)
            .map_err(Error::from)
    }

    pub async fn scan<'a>(
        &'a self,
        services: &'a [Uuid],
    ) -> Result<impl Stream<Item = AdvertisingDevice> + Send + Unpin + 'a> {
        Ok(self.0.scan(services).await?.map(|adv| AdvertisingDevice {
            device: Device(DeviceImpl(adv.device)),
            adv_data: AdvertisementData {
                local_name: adv.adv_data.local_name,
                manufacturer_data: adv.adv_data.manufacturer_data.map(|man| ManufacturerData {
                    company_id: man.company_id,
                    data: man.data,
                }),
                services: adv.adv_data.services,
                service_data: adv.adv_data.service_data,
                tx_power_level: adv.adv_data.tx_power_level,
                is_connectable: adv.adv_data.is_connectable,
            },
            rssi: adv.rssi,
        }))
    }

    pub async fn discover_devices<'a>(
        &'a self,
        services: &'a [Uuid],
    ) -> Result<impl Stream<Item = Result<Device>> + Send + Unpin + 'a> {
        Ok(self
            .0
            .discover_devices(services)
            .await?
            .map(|dev| dev.map(|dev| Device(DeviceImpl(dev))).map_err(Error::from)))
    }

    pub async fn connect_device(&self, device: &Device) -> Result<()> {
        self.0.connect_device(&device.0 .0).await.map_err(Error::from)
    }

    pub async fn disconnect_device(&self, device: &Device) -> Result<()> {
        self.0.disconnect_device(&device.0 .0).await.map_err(Error::from)
    }

    pub async fn device_connection_events<'a>(
        &'a self,
        device: &'a Device,
    ) -> Result<impl Stream<Item = ConnectionEvent> + Send + Unpin + 'a> {
        Ok(self.0.device_connection_events(&device.0 .0).await?.map(|e| match e {
            android_ble::ConnectionEvent::Connected => ConnectionEvent::Connected,
            android_ble::ConnectionEvent::Disconnected => ConnectionEvent::Disconnected,
        }))
    }
}

fn convert_devices(src: Vec<android_ble::Device>) -> Vec<Device> {
    src.into_iter().map(|dev| Device(DeviceImpl(dev))).collect()
}
