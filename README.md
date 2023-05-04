# unzip_lib
Very simple unzip library for Rust language

Example:

```
use std::io::Error;
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
