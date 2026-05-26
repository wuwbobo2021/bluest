use crate::{Error, Result, Uuid};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DescriptorImpl(pub(super) android_ble::Descriptor);

impl DescriptorImpl {
    pub fn uuid(&self) -> Uuid {
        self.0.uuid()
    }

    pub async fn uuid_async(&self) -> Result<Uuid> {
        Ok(self.uuid())
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
}
