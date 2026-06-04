use futures_core::Stream;
use futures_lite::StreamExt;
use uuid::Uuid;

use crate::android::service::ServiceImpl;
use crate::error::ErrorKind;
use crate::pairing::PairingAgent;
use crate::{DeviceId, Error, Result, Service, ServicesChanged};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DeviceImpl(pub(super) android_ble::Device);

impl std::fmt::Display for DeviceImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl DeviceImpl {
    pub fn id(&self) -> DeviceId {
        self.0.id()
    }

    pub fn name(&self) -> Result<String> {
        self.0.name().map_err(Error::from)
    }

    pub async fn name_async(&self) -> Result<String> {
        self.name()
    }

    pub async fn is_connected(&self) -> bool {
        self.0.is_connected().await
    }

    pub async fn is_paired(&self) -> Result<bool> {
        self.0.is_paired().await.map_err(Error::from)
    }

    pub async fn pair(&self) -> Result<()> {
        self.0.pair().await.map_err(Error::from)
    }

    pub async fn pair_with_agent<T: PairingAgent + 'static>(&self, _agent: &T) -> Result<()> {
        Err(Error::new(
            ErrorKind::NotSupported,
            None,
            "Android does not support custom pairing agent",
        ))
    }

    pub async fn unpair(&self) -> Result<()> {
        Err(Error::new(
            ErrorKind::NotSupported,
            None,
            "Android might not allow bluetooth device unpairing in an application",
        ))
    }

    pub async fn discover_services(&self) -> Result<Vec<Service>> {
        self.0
            .discover_services()
            .await
            .map(convert_services)
            .map_err(Error::from)
    }

    pub async fn discover_services_with_uuid(&self, uuid: Uuid) -> Result<Vec<Service>> {
        self.0
            .discover_services_with_uuid(uuid)
            .await
            .map(convert_services)
            .map_err(Error::from)
    }

    pub async fn services(&self) -> Result<Vec<Service>> {
        self.0.services().await.map(convert_services).map_err(Error::from)
    }

    pub async fn service_changed_indications(
        &self,
    ) -> Result<impl Stream<Item = Result<ServicesChanged>> + Send + Unpin + '_> {
        Ok(self.0.service_changed_indications().await?.map(|ch| {
            ch.map(|ch| ServicesChanged(ServicesChangedImpl(ch)))
                .map_err(Error::from)
        }))
    }

    pub async fn rssi(&self) -> Result<i16> {
        self.0.rssi().await.map_err(Error::from)
    }

    #[cfg(feature = "l2cap")]
    pub async fn open_l2cap_channel(&self, psm: u16, secure: bool) -> Result<super::l2cap_channel::L2capChannel> {
        self.0
            .open_l2cap_channel(psm, secure)
            .await
            .map(|ch| ch.split())
            .map(|(reader, writer)| super::l2cap_channel::L2capChannel { reader, writer })
            .map_err(Error::from)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ServicesChangedImpl(android_ble::ServicesChanged);

impl ServicesChangedImpl {
    pub fn was_invalidated(&self, service: &Service) -> bool {
        self.0.was_invalidated(&service.0 .0)
    }
}

pub(super) fn convert_services(src: Vec<android_ble::Service>) -> Vec<Service> {
    src.into_iter().map(|ser| Service(ServiceImpl(ser))).collect()
}
