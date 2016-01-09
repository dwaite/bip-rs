//! Iterators over various pieces of information within a metainfo file.

use bip_util::sha::{self};

use metainfo::{File};

/// Iterator over each File within the MetainfoFile.
pub struct Files<'a> {
    index: usize,
    files: &'a [File]
}

impl<'a> Files<'a> {
    pub fn new(files: &'a [File]) -> Files<'a> {
        Files{ index: 0, files: files }
    }
}

impl<'a> Iterator for Files<'a> {
    type Item = &'a File;
    
    fn next(&mut self) -> Option<&'a File> {
        if let Some(file) = self.files.get(self.index) {
            self.index += 1;
            Some(file)
        } else {
            None
        }
    }
}

//----------------------------------------------------------------------------//

/// Iterator over each piece hash within the MetainfoFile.
pub struct Pieces<'a> {
    index:  usize,
    pieces: &'a [[u8; sha::SHA_HASH_LEN]]
}

impl<'a> Pieces<'a> {
    pub fn new(pieces: &'a [[u8; sha::SHA_HASH_LEN]]) -> Pieces<'a> {
        Pieces{ index: 0, pieces: pieces }
    }
}

impl<'a> Iterator for Pieces<'a> {
    type Item = &'a [u8];
    
    fn next(&mut self) -> Option<&'a [u8]> {
        if let Some(hash) = self.pieces.get(self.index) {
            self.index += 1;
            Some(hash)
        } else {
            None
        }
    }
}

//----------------------------------------------------------------------------//

/// Iterator over each file path element within the MetainfoFile.
pub struct Paths<'a> {
    paths:  &'a [String],
    index: usize
}

impl<'a> Paths<'a> {
    pub fn new(paths: &'a [String]) -> Paths<'a> {
        Paths{ index: 0, paths: paths }
    }
}

impl<'a> Iterator for Paths<'a> {
    type Item = &'a str;
    
    fn next(&mut self) -> Option<&'a str> {
        if let Some(paths) = self.paths.get(self.index) {
            self.index += 1;
            Some(&paths)
        } else {
            None
        }
    }
}