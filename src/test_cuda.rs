use cudarc::driver::{CudaDevice, DriverError};

fn test() {
    let count = cudarc::driver::CudaDevice::count().unwrap();
    let dev = CudaDevice::new(0).unwrap();
}
