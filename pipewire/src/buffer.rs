use super::stream::Stream;

#[cfg(feature = "v1_0_8")]
use spa::buffer::MetaSyncTimeline;
use spa::buffer::{Data, Meta, MetaRegion, MetaType};

use std::convert::TryFrom;
use std::ptr::NonNull;

pub struct Buffer<'s> {
    buf: NonNull<pw_sys::pw_buffer>,

    /// In Pipewire, buffers are owned by the stream that generated them.
    /// This reference ensures that this rule is respected.
    stream: &'s Stream,
}

impl Buffer<'_> {
    pub(crate) unsafe fn from_raw(
        buf: *mut pw_sys::pw_buffer,
        stream: &Stream,
    ) -> Option<Buffer<'_>> {
        NonNull::new(buf).map(|buf| Buffer { buf, stream })
    }

    pub fn datas_mut(&mut self) -> &mut [Data] {
        let buffer: *mut spa_sys::spa_buffer = unsafe { self.buf.as_ref().buffer };

        let slice_of_data = if !buffer.is_null()
            && unsafe { (*buffer).n_datas > 0 && !(*buffer).datas.is_null() }
        {
            unsafe {
                let datas = (*buffer).datas as *mut Data;
                std::slice::from_raw_parts_mut(datas, usize::try_from((*buffer).n_datas).unwrap())
            }
        } else {
            &mut []
        };

        slice_of_data
    }

    pub fn metas_mut(&mut self) -> &mut [Meta] {
        let buffer: *mut spa_sys::spa_buffer = unsafe { self.buf.as_ref().buffer };

        if !buffer.is_null() && unsafe { (*buffer).n_metas > 0 && !(*buffer).metas.is_null() } {
            unsafe {
                let metas = (*buffer).metas as *mut Meta;
                std::slice::from_raw_parts_mut(metas, usize::try_from((*buffer).n_metas).unwrap())
            }
        } else {
            &mut []
        }
    }

    pub fn find_meta_mut(&mut self, meta_type: MetaType) -> Option<&mut Meta> {
        self.metas_mut().iter_mut().find(|m| m.type_() == meta_type)
    }

    pub fn video_crop(&mut self) -> Option<&MetaRegion> {
        self.find_meta_mut(MetaType::VideoCrop)?.video_crop()
    }

    #[cfg(feature = "v1_0_8")]
    pub fn sync_timeline(&mut self) -> Option<&MetaSyncTimeline> {
        self.find_meta_mut(MetaType::SyncTimeline)?.sync_timeline()
    }

    #[cfg(feature = "v0_3_49")]
    pub fn requested(&self) -> u64 {
        unsafe { self.buf.as_ref().requested }
    }
}

impl Drop for Buffer<'_> {
    fn drop(&mut self) {
        unsafe {
            self.stream.queue_raw_buffer(self.buf.as_ptr());
        }
    }
}
