# unzip_lib
Very simple unzip library for Rust language

Example1:

```
use std::io::{Error, ErrorKind};
use std::time::Instant;
use unzip_lib::{FileProcessor, open_zip_archive};

struct DataCollector {
}

impl FileProcessor for DataCollector {
    fn set_file(&mut self, file_name: &String, file_size: usize) -> Result<bool, Error> {
        // check file name and size and return error or true if file should be processed or false when file should not be processed
        Ok(true)
    }

    fn process_file(&mut self, data: Vec<u8>)  -> Result<(), Error> {
        // process file data somehow
        Ok(())
    }

    fn add_file(&mut self, _file: ZipFile) -> Result<(), Error> {
        Err(Error::new(ErrorKind::Unsupported, "Unsupported"))
    }
}

fn main() -> Result<(), Error> {
    let file_name = &std::env::args().nth(1).ok_or(Error::new(ErrorKind::NotFound, "no file name given"))?;
    let start = Instant::now();
    let mut zip_archive = open_zip_archive(file_name)?;
    let mut data_collector = DataCollector{};
    zip_archive.process_files(&mut data_collector)?;
    println!("Elapsed time {} ms", start.elapsed().as_millis());
    Ok(())
}
```

Example2:

```
use std::io::{Error, ErrorKind};
use std::time::Instant;
use unzip_lib::{FileProcessor, open_zip_archive};

struct DataCollector {
}

impl FileProcessor for DataCollector {
    fn set_file(&mut self, _file_name: &String, _file_size: usize) -> Result<bool, Error> {
        Err(Error::new(ErrorKind::Unsupported, "Unsupported"))
    }

    fn process_file(&mut self, _file_data: Vec<u8>) -> Result<(), Error> {
        Err(Error::new(ErrorKind::Unsupported, "Unsupported"))
    }

    fn add_file(&mut self, file: ZipFile) -> Result<(), Error> {
        //save file object somehow
        Ok(())
    }
}

impl DataCollector {
    fn process_file_data(&self, archive: &mut ZipArchive) -> Result<(), Error> {
        //get file data using seek_and_decompress
        //archive.seek_and_decompress(file.as_ref().unwrap())
        //then process file data somehow
        Ok(())
    }
}

fn main() -> Result<(), Error> {
    let file_name = &std::env::args().nth(1).ok_or(Error::new(ErrorKind::NotFound, "no file name given"))?;
    let start = Instant::now();
    let mut zip_archive = open_zip_archive(file_name)?;
    let mut data_collector = DataCollector2{};
    zip_archive.process_files_for_later(&mut data_collector)?;
    data_collector.process_file_data(&mut zip_archive)?;
    println!("Elapsed time {} ms", start.elapsed().as_millis());
    Ok(())
}
```
