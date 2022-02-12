use std::ops::Deref;
use ogg_sys::{
    ogg_stream_state,
    ogg_packet,
    ogg_page,
    ogg_stream_init,
    ogg_stream_clear,
    ogg_stream_packetin,
    ogg_stream_flush,
    ogg_stream_pageout
};

const STREAM_ID: i32 = 888765668; //cool id

pub struct OggStream {
    ogg: *mut ogg_stream_state,
    counter: i64,
    samples: i64,
    buffer: Vec<u8>
}

unsafe impl Send for OggStream {}
unsafe impl Sync for OggStream {}

fn write_page(page: &ogg_page, buffer: &mut Vec<u8>) -> bool {
    if page.header.is_null() {
        return false
    }

    unsafe {
        let header = std::slice::from_raw_parts(page.header, page.header_len as usize);
        let body = std::slice::from_raw_parts(page.body, page.body_len as usize);

        buffer.extend_from_slice(header);
        buffer.extend_from_slice(body);

        true
    }
}

//literally c
impl OggStream {

    pub fn new() -> Self {
        unsafe {
            let ogg = Box::into_raw(Box::<ogg_stream_state>::new_zeroed().assume_init());
            ogg_stream_init(ogg, STREAM_ID);

            Self {
                ogg,
                counter: 0,
                samples: 0,
                buffer: Vec::new()
            }
        }
    }

    pub fn put(&mut self, data: &[u8], samples: u64) {
        unsafe {
            self.samples = self.samples.wrapping_add(samples as i64);

            let mut packet = ogg_packet {
                packet: data.as_ptr() as *mut u8,
                bytes: data.len() as i32,
                b_o_s: if self.counter == 0 { 1 } else { 0 },
                e_o_s: 0,
                granulepos: self.samples,
                packetno: self.counter
            };

            self.counter = self.counter.wrapping_add(1);

            ogg_stream_packetin(self.ogg, (&mut packet) as *mut _);
        }
    }

    pub fn flush(&mut self) {
        unsafe {
            loop {
                let mut data: ogg_page = std::mem::zeroed();
                ogg_stream_flush(self.ogg, (&mut data) as *mut _);
                if !write_page(&data, &mut self.buffer) {
                    break
                }
            }
        }
    }

    pub fn take(&mut self) -> OggSlice {
        unsafe {
            loop {
                let mut data: ogg_page = std::mem::zeroed();
                ogg_stream_pageout(self.ogg, (&mut data) as *mut _);
                if !write_page(&data, &mut self.buffer) {
                    break
                }
            }

            OggSlice {
                buffer: &mut self.buffer
            }
        }
    }
}

impl Drop for OggStream {
    fn drop(&mut self) {
        unsafe {
            ogg_stream_clear(self.ogg);
            drop(Box::from_raw(self.ogg));
        }
    }
}

pub struct OggSlice<'a> {
    buffer: &'a mut Vec<u8>
}

impl<'a> Deref for OggSlice<'a> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl<'a> Drop for OggSlice<'a> {
    fn drop(&mut self) {
        self.buffer.clear();
    }
}