use crate::{ux::*, SquashNumber};

pub trait SquashUint: SquashNumber {}
impl SquashUint for u8 {}
impl SquashUint for u16 {}
impl SquashUint for u24 {}
impl SquashUint for u32 {}
impl SquashUint for u40 {}
impl SquashUint for u48 {}
impl SquashUint for u56 {}
impl SquashUint for u64 {}
