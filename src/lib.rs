//! example:
//! ```
//! #[mmio(u32, base)]
//! pub struct GLB {
//!     #[offset(0x40000180)]
//!     pub GpioInput: gpio::input::Cache,
//! }
//! ```
//! GLB 为寄存器配置表（或者称之为寄存器组），32bit 寄存器。
//! 里面包含寄存器 GpioInput，偏移地址为 0x40000180
//!
//! mmio 表示寄存器为内存映射的寄存器，可以直接通过内存地址访问。
//!
//! Cache 一定需要是 Tuple(u32) 类型，即有如下定义：
//! ```
//! pub mod gpio {
//!     pub mod input {
//!         pub struct Cache(pub(crate) u32);
//!     }
//! }
//! ```
//! base 为 GLB 配置表的基地址，如果没有 base，则默认为 0.
//! base::<dyn> 表示基地址为可配置的（重映射），这种情况一般是可以通过读取某个寄存器来获取基地址。
//! 但是我们希望缓存这样的地址，因此将 GLB 更改为类型  `struct GLB(usize);` GLB.0 为当前基地址。
//!
//! 不论是可重映射 profile 还是静态固定的 profile，寄存器的 offset 都是必须的。
//!
//! 最终的地址计算永远是 base + offset。如果 base 是高 n bits，则需要将低 bit 用 0 补全。
//!

#![allow(dead_code)]
mod profile;
mod register;

use profile::{Profile, ProfileAttr, ProfileItem};
use quote::quote;
#[proc_macro_attribute]
pub fn mmio(
    _attr: proc_macro::TokenStream,
    _item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let profile_attr = syn::parse::<ProfileAttr>(_attr);
    if let Err(err) = profile_attr {
        return err.to_compile_error().into();
    }
    let profile_item = syn::parse::<ProfileItem>(_item);
    if let Err(err) = profile_item {
        return err.to_compile_error().into();
    }
    let profile = Profile {
        attr: profile_attr.unwrap(),
        item: profile_item.unwrap(),
    };
    quote!(#profile).into()
}
