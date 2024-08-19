bitflags!(
    /// Represents the different kind of shadows that can be cast
    /// onto a floor tile.
    pub struct ShadowFlags: i32 {
        /// North edge
        const N_EDGE = 0x1;
        /// South edge
        const S_EDGE = 0x2;
        /// East edge
        const E_EDGE = 0x4;
        /// West edge
        const W_EDGE = 0x8;
        /// North East corner
        const NE_CORNER = 0x10;
        /// North West corner
        const NW_CORNER = 0x20;
        /// South East corner
        const SE_CORNER = 0x40;
        /// South West corner
        const SW_CORNER = 0x80;
    }
);
