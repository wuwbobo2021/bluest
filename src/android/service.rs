use super::device::convert_services;
use crate::android::characteristic::CharacteristicImpl;
use crate::{Characteristic, Error, Result, Service, Uuid};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ServiceImpl(pub(super) android_ble::Service);

impl ServiceImpl {
    pub fn uuid(&self) -> Uuid {
        self.0.uuid()
    }

    pub async fn uuid_async(&self) -> Result<Uuid> {
        Ok(self.uuid())
    }

    pub async fn is_primary(&self) -> Result<bool> {
        self.0.is_primary().await.map_err(Error::from)
    }

    pub async fn discover_characteristics(&self) -> Result<Vec<Characteristic>> {
        self.0
            .discover_characteristics()
            .await
            .map(convert_chars)
            .map_err(Error::from)
    }

    pub async fn discover_characteristics_with_uuid(&self, uuid: Uuid) -> Result<Vec<Characteristic>> {
        self.0
            .discover_characteristics_with_uuid(uuid)
            .await
            .map(convert_chars)
            .map_err(Error::from)
    }

    pub async fn characteristics(&self) -> Result<Vec<Characteristic>> {
        self.0.characteristics().await.map(convert_chars).map_err(Error::from)
    }

    pub async fn discover_included_services(&self) -> Result<Vec<Service>> {
        self.0
            .discover_included_services()
            .await
            .map(convert_services)
            .map_err(Error::from)
    }

    pub async fn discover_included_services_with_uuid(&self, uuid: Uuid) -> Result<Vec<Service>> {
        self.0
            .discover_included_services_with_uuid(uuid)
            .await
            .map(convert_services)
            .map_err(Error::from)
    }

    pub async fn included_services(&self) -> Result<Vec<Service>> {
        self.0
            .included_services()
            .await
            .map(convert_services)
            .map_err(Error::from)
    }
}

fn convert_chars(src: Vec<android_ble::Characteristic>) -> Vec<Characteristic> {
    src.into_iter()
        .map(|ch| Characteristic(CharacteristicImpl(ch)))
        .collect()
}
