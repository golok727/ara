use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum GpuContextCreateError {
    #[error("Error Creating Context: No suitable adapter found")]
    AdapterMissing,
    #[error("Error Creating Context: ({0})")]
    RequestDeviceError(wgpu::RequestDeviceError),
    #[error("Error Creating Context:  ({0})")]
    RequestAdapterError(wgpu::RequestAdapterError),
}
