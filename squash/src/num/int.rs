use crate::{ux::*, SquashNumber};

pub trait SquashInteger: SquashNumber {}
impl SquashInteger for i8 {}
impl SquashInteger for i16 {}
impl SquashInteger for i24 {}
impl SquashInteger for i32 {}
impl SquashInteger for i40 {}
impl SquashInteger for i48 {}
impl SquashInteger for i56 {}
impl SquashInteger for i64 {}
