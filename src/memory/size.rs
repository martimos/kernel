pub enum Size {
    KiB(usize),
    MiB(usize),
    GiB(usize),
    TiB(usize),
}

impl Size {
    pub const fn bytes(self) -> usize {
        match self {
            Size::KiB(x) => x * 1024,
            Size::MiB(x) => x * 1024 * 1024,
            Size::GiB(x) => x * 1024 * 1024 * 1024,
            Size::TiB(x) => x * 1024 * 1024 * 1024 * 1024,
        }
    }
}
