use futures_core::Stream;
use futures_lite::StreamExt;
use uuid::Uuid;

use crate::android::descriptor::DescriptorImpl;
use crate::{CharacteristicProperties, Descriptor, Error, Result};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CharacteristicImpl(pub(super) android_ble::Characteristic);

impl CharacteristicImpl {
    pub fn uuid(&self) -> Uuid {
        self.0.uuid()
    }

    pub async fn uuid_async(&self) -> Result<Uuid> {
        Ok(self.uuid())
    }

    pub async fn properties(&self) -> Result<CharacteristicProperties> {
        self.0
            .properties()
            .await
            .map(|props| CharacteristicProperties {
                broadcast: props.broadcast,
                read: props.read,
                write_without_response: props.write_without_response,
                write: props.write,
                notify: props.notify,
                indicate: props.indicate,
                authenticated_signed_writes: props.authenticated_signed_writes,
                extended_properties: props.extended_properties,
                reliable_write: props.reliable_write,
                writable_auxiliaries: props.writable_auxiliaries,
            })
            .map_err(Error::from)
    }

    pub async fn value(&self) -> Result<Vec<u8>> {
        self.0.value().await.map_err(Error::from)
    }

    pub async fn read(&self) -> Result<Vec<u8>> {
        self.0.read().await.map_err(Error::from)
    }

    pub async fn write(&self, value: &[u8]) -> Result<()> {
        self.0.write(value).await.map_err(Error::from)
    }

    pub async fn write_without_response(&self, value: &[u8]) -> Result<()> {
        self.0.write_without_response(value).await.map_err(Error::from)
    }

    pub fn max_write_len(&self) -> Result<usize> {
        self.0.max_write_len().map_err(Error::from)
    }

    pub async fn max_write_len_async(&self) -> Result<usize> {
        self.max_write_len()
    }

    pub async fn notify(&self) -> Result<impl Stream<Item = Result<Vec<u8>>> + Send + Unpin + '_> {
        Ok(self.0.notify().await?.map(|item| item.map_err(Error::from)))
    }

    pub async fn is_notifying(&self) -> Result<bool> {
        self.0.is_notifying().await.map_err(Error::from)
    }

    pub async fn discover_descriptors(&self) -> Result<Vec<Descriptor>> {
        self.descriptors().await
    }

    pub async fn descriptors(&self) -> Result<Vec<Descriptor>> {
        self.0
            .descriptors()
            .await
            .map(|descs| descs.into_iter().map(|desc| Descriptor(DescriptorImpl(desc))).collect())
            .map_err(Error::from)
    }
}
