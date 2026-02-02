// Copyright The pipewire-rs Contributors.
// SPDX-License-Identifier: MIT

use std::fmt::Debug;

use crate::utils::Region;

/// Type of metadata attached to a buffer.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct MetaType(spa_sys::spa_meta_type);

#[allow(non_upper_case_globals)]
impl MetaType {
    pub const Invalid: Self = Self(spa_sys::SPA_META_Invalid);
    pub const Header: Self = Self(spa_sys::SPA_META_Header);
    pub const VideoCrop: Self = Self(spa_sys::SPA_META_VideoCrop);
    pub const VideoDamage: Self = Self(spa_sys::SPA_META_VideoDamage);
    pub const Bitmap: Self = Self(spa_sys::SPA_META_Bitmap);
    pub const Cursor: Self = Self(spa_sys::SPA_META_Cursor);
    pub const Control: Self = Self(spa_sys::SPA_META_Control);
    #[cfg(feature = "v0_3_21")]
    pub const Busy: Self = Self(spa_sys::SPA_META_Busy);
    #[cfg(feature = "v0_3_62")]
    pub const VideoTransform: Self = Self(spa_sys::SPA_META_VideoTransform);
    #[cfg(feature = "v1_0_8")]
    pub const SyncTimeline: Self = Self(spa_sys::SPA_META_SyncTimeline);

    pub fn from_raw(raw: spa_sys::spa_meta_type) -> Self {
        Self(raw)
    }

    pub fn as_raw(&self) -> spa_sys::spa_meta_type {
        self.0
    }

    /// Returns the expected size in bytes for this metadata type.
    ///
    /// Returns `None` for unknown or variable-size metadata types.
    pub fn size(&self) -> Option<i32> {
        match *self {
            Self::Header => Some(std::mem::size_of::<spa_sys::spa_meta_header>() as i32),
            Self::VideoCrop => Some(std::mem::size_of::<spa_sys::spa_meta_region>() as i32),
            #[cfg(feature = "v0_3_21")]
            Self::Busy => Some(std::mem::size_of::<spa_sys::spa_meta_busy>() as i32),
            #[cfg(feature = "v0_3_62")]
            Self::VideoTransform => {
                Some(std::mem::size_of::<spa_sys::spa_meta_videotransform>() as i32)
            }
            #[cfg(feature = "v1_0_8")]
            Self::SyncTimeline => {
                Some(std::mem::size_of::<spa_sys::spa_meta_sync_timeline>() as i32)
            }
            _ => None,
        }
    }
}

impl Debug for MetaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match *self {
            Self::Invalid => "MetaType::Invalid",
            Self::Header => "MetaType::Header",
            Self::VideoCrop => "MetaType::VideoCrop",
            Self::VideoDamage => "MetaType::VideoDamage",
            Self::Bitmap => "MetaType::Bitmap",
            Self::Cursor => "MetaType::Cursor",
            Self::Control => "MetaType::Control",
            #[cfg(feature = "v0_3_21")]
            Self::Busy => "MetaType::Busy",
            #[cfg(feature = "v0_3_62")]
            Self::VideoTransform => "MetaType::VideoTransform",
            #[cfg(feature = "v1_0_8")]
            Self::SyncTimeline => "MetaType::SyncTimeline",
            _ => "MetaType::Unknown",
        };
        f.write_str(name)
    }
}

/// Metadata attached to a buffer.
#[repr(transparent)]
pub struct Meta(spa_sys::spa_meta);

impl Meta {
    pub fn as_raw(&self) -> &spa_sys::spa_meta {
        &self.0
    }

    pub fn type_(&self) -> MetaType {
        MetaType::from_raw(self.0.type_)
    }

    pub fn size(&self) -> u32 {
        self.0.size
    }

    pub fn data(&self) -> *mut std::ffi::c_void {
        self.0.data
    }

    pub fn video_crop(&self) -> Option<&MetaRegion> {
        if self.type_() == MetaType::VideoCrop
            && self.size() >= std::mem::size_of::<spa_sys::spa_meta_region>() as u32
        {
            unsafe { Some(&*(self.0.data as *const MetaRegion)) }
        } else {
            None
        }
    }
}

impl Debug for Meta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Meta")
            .field("type", &self.type_())
            .field("size", &self.size())
            .finish()
    }
}

#[cfg(feature = "v1_0_8")]
mod sync_timeline_impl {
    use super::*;

    /// Sync timeline metadata for explicit synchronization.
    #[derive(Clone)]
    #[repr(transparent)]
    pub struct MetaSyncTimeline(pub(super) spa_sys::spa_meta_sync_timeline);

    impl MetaSyncTimeline {
        pub fn as_raw(&self) -> &spa_sys::spa_meta_sync_timeline {
            &self.0
        }

        pub fn acquire_point(&self) -> u64 {
            self.0.acquire_point
        }

        pub fn release_point(&self) -> u64 {
            self.0.release_point
        }

        pub fn flags(&self) -> u32 {
            self.0.flags
        }
    }

    impl Debug for MetaSyncTimeline {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("MetaSyncTimeline")
                .field("acquire_point", &self.0.acquire_point)
                .field("release_point", &self.0.release_point)
                .field("flags", &self.0.flags)
                .finish()
        }
    }

    impl Meta {
        pub fn sync_timeline(&self) -> Option<&MetaSyncTimeline> {
            if self.type_() == MetaType::SyncTimeline
                && self.size() >= std::mem::size_of::<spa_sys::spa_meta_sync_timeline>() as u32
            {
                unsafe { Some(&*(self.0.data as *const MetaSyncTimeline)) }
            } else {
                None
            }
        }
    }
}

#[cfg(feature = "v1_0_8")]
pub use sync_timeline_impl::MetaSyncTimeline;

/// A region metadata, used for video crop and video damage.
#[derive(Clone)]
#[repr(transparent)]
pub struct MetaRegion(spa_sys::spa_meta_region);

impl MetaRegion {
    pub fn as_raw(&self) -> &spa_sys::spa_meta_region {
        &self.0
    }

    pub fn region(&self) -> &Region {
        &self.0.region
    }

    pub fn is_valid(&self) -> bool {
        self.0.region.size.width > 0 && self.0.region.size.height > 0
    }
}

impl Debug for MetaRegion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MetaRegion")
            .field("x", &self.0.region.position.x)
            .field("y", &self.0.region.position.y)
            .field("width", &self.0.region.size.width)
            .field("height", &self.0.region.size.height)
            .finish()
    }
}
