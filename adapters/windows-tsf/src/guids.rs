// SPDX-License-Identifier: GPL-3.0-or-later
//! GUIDs và hằng số chính thức của TextVN TSF (P1-1 §1).

#[cfg(windows)]
use windows::core::GUID;

#[cfg(windows)]
pub const CLSID_TEXTVN_TIP: GUID = GUID::from_u128(0x6F2B9C31_8E47_4D2A_9C84_1D5A3E70F9B8);
#[cfg(windows)]
pub const PROFILE_TEXTVN: GUID = GUID::from_u128(0xC4A91F52_77B3_4E19_8A6D_2F8C0B6E5A13);
#[cfg(windows)]
pub const DISPATTR_TEXTVN: GUID = GUID::from_u128(0x9D2E7A44_1B5C_4F63_A086_33D15E4C7B21);
pub const LANGID_VI: u16 = 0x042A; // vi-VN
pub const LANGID_EN: u16 = 0x0409; // en-US
pub const GUID_PRESERVED_TOGGLE: GUID = GUID::from_u128(0x7A5B8C2D_3E4F_4A1B_9C8D_5E6F7A8B9C0D);
