pub enum KernelError {
    BufferInitError,
}

pub type KernelResult<T> = Result<T, KernelError>;
