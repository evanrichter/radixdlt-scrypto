// Turn on `no_std`
#![cfg_attr(not(feature = "std"), no_std)]

/// Defines system function and component call protocol.
pub mod abi;
/// A module that handles buffer encoding and decoding.
pub mod buffer;
/// Scrypto higher level abstraction.
pub mod constructs;
/// A module that facilitates system calls.
pub mod kernel;
/// A library of common routines.
pub mod library;
/// Scrypto primitive types.
pub mod types;
/// Utility functions, such as hashing and hex decoding.
pub mod utils;

// Re-export scrypto derive.
#[cfg(feature = "derive")]
#[allow(unused_imports)]
#[macro_use]
extern crate scrypto_derive;
#[cfg(feature = "derive")]
pub use scrypto_derive::*;

/// Call a method of a blueprint.
#[macro_export]
macro_rules! call_blueprint {
    ($rtn_type: ty, $blueprint: expr, $component: expr, $method: expr $(,)?) => {
        {
            extern crate alloc;
            let rtn = scrypto::constructs::Blueprint::call(&$blueprint, $component, $method, alloc::vec::Vec::new());
            scrypto::buffer::radix_decode::<$rtn_type>(&rtn)
        }
    };

    ($rtn_type: ty, $blueprint: expr, $component: expr, $method: expr, $($args: expr),+ $(,)?) => {
        {
            extern crate alloc;
            let mut args = alloc::vec::Vec::new();
            $(args.push(scrypto::buffer::radix_encode(&$args));)+
            let rtn = scrypto::constructs::Blueprint::call(&$blueprint, $component, $method, args);
            scrypto::buffer::radix_decode::<$rtn_type>(&rtn)
        }
    };
}

/// Call a method of a component.
#[macro_export]
macro_rules! call_component {
    ($rtn_type: ty, $component: expr, $method: expr $(,)?) => {
        {
            extern crate alloc;
            let rtn = scrypto::constructs::Component::call(&$component, $method, alloc::vec::Vec::new());
            scrypto::buffer::radix_decode::<$rtn_type>(&rtn)
        }
    };

    ($rtn_type: ty, $component: expr, $method: expr, $($args: expr),+ $(,)?) => {
        {
            extern crate alloc;
            let mut args = alloc::vec::Vec::new();
            $(args.push(scrypto::buffer::radix_encode(&$args));)+
            let rtn = scrypto::constructs::Component::call(&$component, $method, args);
            scrypto::buffer::radix_decode::<$rtn_type>(&rtn)
        }
    };
}

/// Log an `ERROR` message.
#[macro_export]
macro_rules! error {
    ($($args: expr),+) => {{
        extern crate alloc;
        scrypto::constructs::Logger::log(scrypto::constructs::LogLevel::Error, alloc::format!($($args),+));
    }};
}

/// Log a `WARN` message.
#[macro_export]
macro_rules! warn {
    ($($args: expr),+) => {{
        extern crate alloc;
        scrypto::constructs::Logger::log(scrypto::constructs::LogLevel::Warn, alloc::format!($($args),+));
    }};
}

/// Log an `INFO` message.
#[macro_export]
macro_rules! info {
    ($($args: expr),+) => {{
        extern crate alloc;
        scrypto::constructs::Logger::log(scrypto::constructs::LogLevel::Info, alloc::format!($($args),+));
    }};
}

/// Log a `DEBUG` message.
#[macro_export]
macro_rules! debug {
    ($($args: expr),+) => {{
        extern crate alloc;
        scrypto::constructs::Logger::log(scrypto::constructs::LogLevel::Debug, alloc::format!($($args),+));
    }};
}

/// Log a `TRACE` message.
#[macro_export]
macro_rules! trace {
    ($($args: expr),+) => {{
        extern crate alloc;
        scrypto::constructs::Logger::log(scrypto::constructs::LogLevel::Trace, alloc::format!($($args),+));
    }};
}
