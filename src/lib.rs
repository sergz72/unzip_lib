use std::fs::File;
use std::io::{BufReader, Error, ErrorKind, Read, Seek, SeekFrom};
use std::mem;
use std::mem::zeroed;
use std::slice::from_raw_parts_mut;
use inflate::inflate_bytes;
use std::fmt::{Display, Formatter};

pub trait FileProcessor {
    fn set_file(&mut self, file_name: &String, file_size: usize) -> Result<bool, Error>;
    fn process_file(&mut self, file_data: Vec<u8>) -> Result<(), Error>;
    fn add_file(&mut self, file: ZipFile) -> Result<(), Error>;
}

#[repr(C, packed)]
struct ZipFileHeader {
    signature: u32,
    version: u16,
    flags: u16,
    compression_method: u16,
    file_time: u16,
    file_date: u16,
    crc: u32,
    compressed_size: u32,
    uncompressed_size: u32,
    file_name_length: u16,
    extra_field_length: u16
}

const ZIP_FILE_HEADER_SIZE: usize = mem::size_of::<ZipFileHeader>();
const COMPRESSION_METHOD_NONE: u16 = 0;
const COMPRESSION_METHOD_DEFLATE: u16 = 8;
const MAX_FILE_NAME_LENGTH: u16 = 512;

impl ZipFileHeader {
    fn print(&self) {
        let signature = self.signature;
        let version = self.version;
        let flags = self.flags;
        let compression_method = self.compression_method;
        println!("ZIP file header:\nSignature: {:#010x}\nVersion: {}\nFlags: {:#06x}\nCompression method: {:#06x}",
                 signature, version, flags, compression_method);
        let crc = self.crc;
        let compressed_size = self.compressed_size;
        let uncompressed_size = self.uncompressed_size;
        let file_name_length = self.file_name_length;
        let extra_field_length = self.extra_field_length;
        println!("CRC: {:#010x}\nCompressed size: {}\nUncompressed size: {}\nFile name length: {}\nExtra field length: {}",
                 crc, compressed_size, uncompressed_size, file_name_length, extra_field_length);
    }

    fn validate(&self) -> Result<bool, Error> {
        if self.signature != 0x04034b50 {
            if self.signature == 0x02014b50 {
                return Ok(false);
            }
            return Err(Error::new(ErrorKind::InvalidInput, "invalid file signature"));
        }
        if self.compression_method != COMPRESSION_METHOD_NONE && self.compression_method != COMPRESSION_METHOD_DEFLATE {
            return Err(Error::new(ErrorKind::InvalidInput, "invalid file compression method"));
        }
        if self.file_name_length > MAX_FILE_NAME_LENGTH {
            return Err(Error::new(ErrorKind::InvalidInput, "file name length is too long"));
        }
        if self.compression_method == COMPRESSION_METHOD_NONE && self.compressed_size != self.uncompressed_size {
            return Err(Error::new(ErrorKind::InvalidInput, "compressed_size != uncompressed_size"));
        }
        Ok(true)
    }
}

pub struct ZipFile {
    header: ZipFileHeader,
    name: String,
    data_offset: u64
}

impl ZipFile {
    pub fn print_header(&self) {
        self.header.print();
    }
    pub fn get_name(&self) -> String { return self.name.clone() }
    pub fn get_size(&self) -> usize { return self.header.uncompressed_size as usize }
}

impl Display for ZipFile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let size = self.header.uncompressed_size;
        write!(f, "Zip file: name {}, size: {}, data offset: {}", self.name, size, self.data_offset)
    }
}

pub struct ZipArchive {
    reader: BufReader<File>,
    current_offset: u64
}

pub fn open_zip_archive(file_name: &String) -> Result<ZipArchive, Error> {
    let file = File::open(file_name)?;
    Ok(ZipArchive{ reader: BufReader::new(file), current_offset: 0 })
}

impl ZipArchive {
    pub fn next_zip_file(&mut self) -> Result<Option<ZipFile>, Error> {
        let mut zip_file = ZipFile { header: unsafe { zeroed() }, name: "".to_string(), data_offset: 0 };
        unsafe {
            let header_slice = from_raw_parts_mut(&mut zip_file.header as *mut _ as *mut u8, ZIP_FILE_HEADER_SIZE);
            self.reader.read_exact(header_slice)?;
            let ok = zip_file.header.validate()?;
            if !ok {
                return Ok(None);
            }
            let mut file_name_vec: Vec<u8> = vec![0; zip_file.header.file_name_length as usize];
            self.reader.read_exact(file_name_vec.as_mut_slice())?;
            zip_file.name = match String::from_utf8(file_name_vec) {
                Ok(n) => n,
                Err(_e) => return Err(Error::new(ErrorKind::InvalidInput, "could not convert file name to string"))
            };
            if zip_file.header.extra_field_length > 0 {
                self.reader.seek_relative(zip_file.header.extra_field_length as i64)?;
            }
            self.current_offset += ZIP_FILE_HEADER_SIZE as u64;
            self.current_offset += zip_file.header.file_name_length as u64;
            self.current_offset += zip_file.header.extra_field_length as u64;
            zip_file.data_offset = self.current_offset
        }
        Ok(Some(zip_file))
    }

    pub fn skip_data(&mut self, file: &ZipFile) -> Result<(), Error> {
        if file.header.compressed_size > 0 {
            self.reader.seek_relative(file.header.compressed_size as i64)?;
            self.current_offset += file.header.compressed_size as u64;
        }
        Ok(())
    }

    pub fn seek_and_decompress(&mut self, zip_file: &ZipFile) -> Result<Vec<u8>, Error> {
        self.reader.seek(SeekFrom::Start(zip_file.data_offset))?;
        self.decompress(zip_file)
    }

    pub fn decompress(&mut self, zip_file: &ZipFile) -> Result<Vec<u8>, Error> {
        if zip_file.header.uncompressed_size == 0 {
            return Ok(Vec::new());
        }
        let mut compressed_data_vec: Vec<u8> = vec![0; zip_file.header.compressed_size as usize];
        self.reader.read_exact(compressed_data_vec.as_mut_slice())?;
        if zip_file.header.compression_method == COMPRESSION_METHOD_NONE {
            return Ok(compressed_data_vec);
        }
        let uncompressed_data_vec = match inflate_bytes(compressed_data_vec.as_slice()) {
            Ok(v) => v,
            Err(e) => return Err(Error::new(ErrorKind::InvalidData, e))
        };
        Ok(uncompressed_data_vec)
    }

    pub fn process_files(&mut self, file_processor: &mut dyn FileProcessor) -> Result<(), Error> {
        loop {
            let maybe_zip_file = self.next_zip_file()?;
            if maybe_zip_file.is_none() {
                break;
            }
            let zip_file = maybe_zip_file.unwrap();
            if zip_file.header.uncompressed_size > 0 {
                if file_processor.set_file(&zip_file.name, zip_file.header.uncompressed_size as usize)? {
                    let data = self.decompress(&zip_file)?;
                    file_processor.process_file(data)?;
                } else {
                    self.skip_data(&zip_file)?;
                }
            }
        }
        Ok(())
    }

    pub fn process_files_for_later(&mut self, file_processor: &mut dyn FileProcessor) -> Result<(), Error> {
        loop {
            let maybe_zip_file = self.next_zip_file()?;
            if maybe_zip_file.is_none() {
                break;
            }
            let zip_file = maybe_zip_file.unwrap();
            if zip_file.header.uncompressed_size > 0 {
                self.skip_data(&zip_file)?;
                file_processor.add_file(zip_file)?;
            }
        }
        Ok(())
    }
}
