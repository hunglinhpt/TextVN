// SPDX-License-Identifier: GPL-3.0-or-later
//! GUIDs và hằng số chính thức của VietIME TSF (P1-1 §1).

#[cfg(windows)]
use windows::core::GUID;

#[cfg(windows)]
pub const CLSID_VIETIME_TIP: GUID = GUID::from_u128(0x6F2B9C31_8E47_4D2A_9C84_1D5A3E70F9B8);
#[cfg(windows)]
pub const PROFILE_VIETIME: GUID = GUID::from_u128(0xC4A91F52_77B3_4E19_8A6D_2F8C0B6E5A13);
#[cfg(windows)]
pub const DISPATTR_VIETIME: GUID = GUID::from_u128(0x9D2E7A44_1B5C_4F63_A086_33D15E4C7B21);
pub const LANGID_VI: u16 = 0x042A; // vi-VN
pub const LANGID_EN: u16 = 0x0409; // en-US
