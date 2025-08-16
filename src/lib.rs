// NOTE: All functions are NOT thread-safe. Serialize access to devices.
use std::ffi::{c_char, CStr, CString, c_void};
use std::fmt;
use std::path::Path;
use std::slice::from_raw_parts;
use std::str::Utf8Error;
use std::sync::Mutex;
use once_cell::sync::Lazy;

static CD_ACCESS_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

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

pub struct STrackMeta {
    pub inner: TrackMeta
}

impl Drop for STrackMeta {
    fn drop(&mut self) {
        unsafe { free_track_metadata(&mut self.inner as *mut TrackMeta) };
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
    // Gets the duration of a track, returns -1 on faliure
    pub fn get_track_duration(devicestr: *const c_char, track: i32) -> i32;
    // Opens a CDStream, returns NULL on failure
    pub fn open_cd_stream(devicestr: *const c_char, track: i32) -> *mut CDStream;
    // Reads from the CD
    pub fn read_cd_stream(stream: *mut CDStream, buffer: *mut c_void, sectors: i32) -> i32;
    // Closes the CDStream
    pub fn close_cd_stream(stream: *mut CDStream);
}

pub fn convert_double_pointer_to_vec(
    data: *mut *mut c_char,
    len: usize,
) -> Result<Vec<String>, Utf8Error> {
    if data.is_null() {
        return Ok(Vec::new());
    }
    
    unsafe {
        from_raw_parts(data, len)
            .iter()
            .map(|ptr| {
                if ptr.is_null() {
                    Ok(String::new())
                } else {
                    CStr::from_ptr(*ptr).to_str().map(ToString::to_string)
                }
            })
            .collect()
    }
}

pub struct SDevList {
    pub inner: Vec<String>,
    original_ptr: *mut *mut c_char,
}

impl Drop for SDevList {
    fn drop(&mut self) {
        if !self.original_ptr.is_null() {
            unsafe {
                free_devices(self.original_ptr);
            }
        }
    }
}

pub fn sget_devices() -> SDevList {
    let _lock = CD_ACCESS_MUTEX.lock().unwrap();
    let devices = unsafe { get_devices() };
    
    // Count devices until null pointer
    let mut len = 0;
    unsafe {
        if !devices.is_null() {
            while !(*devices.add(len)).is_null() {
                len += 1;
            }
        }
    }

    let sdevices: Vec<String> = if len > 0 {
        convert_double_pointer_to_vec(devices, len)
            .expect("Failed to convert char** to Vec<String>")
    } else {
        Vec::new()
    };
    
    SDevList {
        inner: sdevices,
        original_ptr: devices,
    }
}

pub fn sverify_audio(device: String) -> bool {
    let _lock = CD_ACCESS_MUTEX.lock().unwrap();
    if !Path::new(&device).exists() { return false };
    return unsafe { verify_audio(CString::new(&*device).expect("Failed to convert to CString").as_ptr())}
}

pub fn strack_num(device: String) -> i32 {
    if !Path::new(&device).exists() {
        return -1;
    }
    let _lock = CD_ACCESS_MUTEX.lock().unwrap();
    unsafe { track_num(CString::new(device).expect("CString conversion failed").as_ptr()) }
}

pub fn sget_track_meta(device: String, track: i32) -> (String, String, String) {
    let _lock = CD_ACCESS_MUTEX.lock().unwrap();
    let c_device = CString::new(device).expect("CString conversion failed");
    let mut meta = unsafe { get_track_metadata(c_device.as_ptr(), track) };
    
    let title = if meta.title.is_null() {
        "Unknown title".to_string()
    } else {
        unsafe { CStr::from_ptr(meta.title).to_string_lossy().into_owned() }
    };
    
    let artist = if meta.artist.is_null() {
        "Unknown artist".to_string()
    } else {
        unsafe { CStr::from_ptr(meta.artist).to_string_lossy().into_owned() }
    };
    
    let genre = if meta.genre.is_null() {
        "Unknown genre".to_string()
    } else {
        unsafe { CStr::from_ptr(meta.genre).to_string_lossy().into_owned() }
    };
    
    unsafe { free_track_metadata(&mut meta) };
    
    (title, artist, genre)
}

pub fn strack_duration(device: String, track: i32) -> i32 {
    let _lock = CD_ACCESS_MUTEX.lock().unwrap();
    return unsafe { get_track_duration(CString::new(device).expect("Failed to convert to CString").as_ptr(), track) };
}

#[repr(C)]
pub struct CDStream {
    _private: [u8; 0],
}

pub struct SCDStream {
    pub inner: *mut CDStream,
}

impl Drop for SCDStream {
    fn drop(&mut self) {
        if !self.inner.is_null() {
            unsafe { close_cd_stream(self.inner) };
        }
    }
}

pub fn sopen_cd_stream(device: &str, track: i32) -> Option<SCDStream> {
    let _lock = CD_ACCESS_MUTEX.lock().unwrap();
    let c_device = CString::new(device).ok()?;
    let stream = unsafe { open_cd_stream(c_device.as_ptr(), track) };
    if stream.is_null() {
        None
    } else {
        Some(SCDStream { inner: stream })
    }
}

pub fn sread_cd_stream(stream: &mut SCDStream, buffer: &mut [u8], sectors: i32) -> i32 {
    let _lock = CD_ACCESS_MUTEX.lock().unwrap();
    unsafe {
        read_cd_stream(
            stream.inner,
            buffer.as_mut_ptr() as *mut c_void,
            sectors,
        )
    }
}
