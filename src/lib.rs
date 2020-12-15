// Copyright (C) 2019 Alibaba Cloud Computing. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 or BSD-3-Clause

//! Virtio Vhost Backend Drivers
//!
//! Virtio devices use virtqueues to transport data efficiently. The first generation of virtqueue
//! is a set of three different single-producer, single-consumer ring structures designed to store
//! generic scatter-gather I/O. The virtio specification 1.1 introduces an alternative compact
//! virtqueue layout named "Packed Virtqueue", which is more friendly to memory cache system and
//! hardware implemented virtio devices. The packed virtqueue uses read-write memory, that means
//! the memory will be both read and written by both host and guest. The new Packed Virtqueue is
//! preferred for performance.
//!
//! Vhost is a mechanism to improve performance of Virtio devices by delegate data plane operations
//! to dedicated IO service processes. Only the configuration, I/O submission notification, and I/O
//! completion interruption are piped through the hypervisor.
//! It uses the same virtqueue layout as Virtio to allow Vhost devices to be mapped directly to
//! Virtio devices. This allows a Vhost device to be accessed directly by a guest OS inside a
//! hypervisor process with an existing Virtio (PCI) driver.
//!
//! The initial vhost implementation is a part of the Linux kernel and uses ioctl interface to
//! communicate with userspace applications. Dedicated kernel worker threads are created to handle
//! IO requests from the guest.
//!
//! Later Vhost-user protocol is introduced to complement the ioctl interface used to control the
//! vhost implementation in the Linux kernel. It implements the control plane needed to establish
//! virtqueues sharing with a user space process on the same host. It uses communication over a
//! Unix domain socket to share file descriptors in the ancillary data of the message.
//! The protocol defines 2 sides of the communication, master and slave. Master is the application
//! that shares its virtqueues. Slave is the consumer of the virtqueues. Master and slave can be
//! either a client (i.e. connecting) or server (listening) in the socket communication.

#![deny(missing_docs)]

#[cfg_attr(
    any(feature = "vhost-user-master", feature = "vhost-user-slave"),
    macro_use
)]
extern crate bitflags;
extern crate libc;
#[cfg(feature = "vhost-kern")]
extern crate vm_memory;
#[cfg_attr(feature = "vhost-kern", macro_use)]
extern crate vmm_sys_util;

mod backend;
pub use backend::*;

#[cfg(feature = "vhost-kern")]
pub mod vhost_kern;
#[cfg(any(feature = "vhost-user-master", feature = "vhost-user-slave"))]
pub mod vhost_user;
#[cfg(feature = "vhost-vsock")]
pub mod vsock;

/// Error codes for vhost operations
#[derive(Debug)]
pub enum Error {
    /// Invalid operations.
    InvalidOperation,
    /// Invalid guest memory.
    InvalidGuestMemory,
    /// Invalid guest memory region.
    InvalidGuestMemoryRegion,
    /// Invalid queue.
    InvalidQueue,
    /// Invalid descriptor table address.
    DescriptorTableAddress,
    /// Invalid used address.
    UsedAddress,
    /// Invalid available address.
    AvailAddress,
    /// Invalid log address.
    LogAddress,
    /// Get features failed.
    VhostGetFeatures,
    /// Set features failed.
    VhostSetFeatures,
    /// Set features failed.
    VhostSetMemTable,
    /// Set vring num failed.
    VhostSetVringNum,
    /// Set vring addr failed.
    VhostSetVringAddr,
    /// Set vring base failed.
    VhostSetVringBase,
    /// Set vring call failed.
    VhostSetVringCall,
    /// Set vring kick failed.
    VhostSetVringKick,
    /// Vhost-net error
    VhostNet(String),
    #[cfg(feature = "vhost-kern")]
    /// Error opening the vhost backend driver.
    VhostOpen(std::io::Error),
    #[cfg(feature = "vhost-kern")]
    /// Error while running ioctl.
    IoctlError(std::io::Error),
    /// Error from IO subsystem.
    IOError(std::io::Error),
    #[cfg(any(feature = "vhost-user-master", feature = "vhost-user-slave"))]
    /// Error from the vhost-user subsystem.
    VhostUserProtocol(vhost_user::Error),
    #[cfg(feature = "vhost-net")]
    /// Set vhost net backend failed.
    VhostNetSetBackend,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::InvalidOperation => write!(f, "invalid vhost operations"),
            Error::InvalidGuestMemory => write!(f, "invalid guest memory object"),
            Error::InvalidGuestMemoryRegion => write!(f, "invalid guest memory region"),
            Error::InvalidQueue => write!(f, "invalid virtque"),
            Error::DescriptorTableAddress => write!(f, "invalid virtque descriptor talbe address"),
            Error::UsedAddress => write!(f, "invalid virtque used talbe address"),
            Error::AvailAddress => write!(f, "invalid virtque available talbe address"),
            Error::LogAddress => write!(f, "invalid virtque log address"),
            Error::VhostGetFeatures => write!(f, "failed to get features"),
            Error::VhostSetFeatures => write!(f, "failed to set features"),
            Error::VhostSetMemTable => write!(f, "failed to set mem table"),
            Error::VhostSetVringNum => write!(f, "failed to set vring num"),
            Error::VhostSetVringAddr => write!(f, "failed to set vring addr"),
            Error::VhostSetVringBase => write!(f, "failed to set vring base"),
            Error::VhostSetVringCall => write!(f, "failed to set vring call"),
            Error::VhostSetVringKick => write!(f, "failed to set vring kick"),
            Error::VhostNet(s) => write!(f, "failed to setup vhost-net: {}", s),
            Error::IOError(e) => write!(f, "IO error: {}", e),
            #[cfg(feature = "vhost-kern")]
            Error::VhostOpen(e) => write!(f, "failure in opening vhost file: {}", e),
            #[cfg(feature = "vhost-kern")]
            Error::IoctlError(e) => write!(f, "failure in vhost ioctl: {}", e),
            #[cfg(any(feature = "vhost-user-master", feature = "vhost-user-slave"))]
            Error::VhostUserProtocol(e) => write!(f, "vhost-user: {}", e),
            #[cfg(feature = "vhost-net")]
            Error::VhostNetSetBackend => write!(f, "failed to set vhost-net backend"),
        }
    }
}

#[cfg(any(feature = "vhost-user-master", feature = "vhost-user-slave"))]
impl std::convert::From<vhost_user::Error> for Error {
    fn from(err: vhost_user::Error) -> Self {
        Error::VhostUserProtocol(err)
    }
}

/// Result of vhost operations
pub type Result<T> = std::result::Result<T, Error>;
