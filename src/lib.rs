// NOTE: All functions are NOT thread-safe. Serialize access to devices.
use std::ffi::{c_char, CStr};
use std::fmt;

#[repr(C)]
/*
 TrackMeta struct expects a title, artist and genre all as strings (c_char as it interacts with C).
 This library proves no wrapper around it as you should only be using TrackMeta given by the C program which has APIs to free.
*/
pub struct TrackMeta {
    pub title: *mut c_char,
    pub artist: *mut c_char,
    pub genre: *mut c_char,
}

impl fmt::Display for TrackMeta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let title = if self.title.is_null() {
            "Unknown title"
        } else {
            unsafe { CStr::from_ptr(self.title) }
                .to_str()
                .unwrap_or("Unknown title")
        };
        let artist = if self.artist.is_null() {
            "Unknown artist"
        } else {
            unsafe { CStr::from_ptr(self.artist) }
                .to_str()
                .unwrap_or("Unknown artist")
        };
        let genre = if self.genre.is_null() {
            "Unknown genre"
        } else {
            unsafe { CStr::from_ptr(self.genre) }
                .to_str()
                .unwrap_or("Unknown genre")
        };
        write!(f, "title: {} artist {} genre {}", title, artist, genre)
    }
}

unsafe extern "C" {
    // Returns an array of strings containing the paths to devices on the system
    pub fn get_devices() -> *mut *mut c_char;
    // Frees the array of devices passed through get_devices
    pub fn free_devices(devices: *mut *mut c_char);
    // Returns if the provided device path is an audio CD
    pub fn verify_audio(devicestr: *const c_char) -> bool;
    // Returns an i32 containing how many tracks are on the CD. Returns -1 on error
    pub fn track_num(devicestr: *const c_char) -> i32;
    // Returns a malloc'd Trackmeta struct containing info about the track
    pub fn get_track_metadata(devicestr: *const c_char, track: i32) -> TrackMeta;
    // Frees the TrackMeta struct returned by get_track_metadata
    pub fn free_track_metadata(meta: *mut TrackMeta);
}
