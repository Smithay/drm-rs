/// The underlying handle for a buffer
pub type RawId = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Id(RawId);

impl Id {
    pub fn from_raw(raw: RawId) -> Self {
        Id(raw)
    }

    pub fn as_raw(&self) -> RawId {
        self.0
    }
}

/// Common functionality of all buffers.
pub trait Buffer {
    /// The width and height of the buffer.
    fn size(&self) -> (u32, u32);
    /// The depth of the buffer.
    fn depth(&self) -> u8;
    /// The number of bits per pixel.
    fn bpp(&self) -> u8;
    /// The pitch of the buffer.
    fn pitch(&self) -> u32;
    /// The GEM handle of the buffer.
    fn handle(&self) -> Id;
}
