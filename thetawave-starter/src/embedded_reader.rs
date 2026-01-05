//! Custom AssetReader that embeds assets from the library at compile time.
//!
//! This uses `include_dir` to embed the entire assets directory into the library,
//! making the base assets portable to downstream games without requiring them to
//! copy assets or set environment variables.

use std::{
    io::Read,
    path::{Path, PathBuf},
    pin::Pin,
    task::Poll,
};

use bevy::asset::io::{AssetReader, AssetReaderError, AssetSource, PathStream, Reader};
use futures_io::{AsyncRead, AsyncSeek};
use futures_lite::Stream;
use include_dir::{include_dir, Dir};

/// The embedded base assets directory.
/// This is embedded at the library's compile time, not the final binary's.
static BASE_ASSETS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../assets");

/// AssetReader that reads from the embedded base assets.
pub struct EmbeddedBaseAssetReader;

impl EmbeddedBaseAssetReader {
    /// Create a new embedded asset reader.
    pub fn new() -> Self {
        Self
    }

    fn get_file(&self, path: &Path) -> Option<&'static [u8]> {
        BASE_ASSETS.get_file(path).map(|f| f.contents())
    }

    fn is_directory(&self, path: &Path) -> bool {
        BASE_ASSETS.get_dir(path).is_some()
    }

    fn read_directory(&self, path: &Path) -> Option<Vec<PathBuf>> {
        BASE_ASSETS.get_dir(path).map(|dir| {
            let mut paths = Vec::new();
            for entry in dir.entries() {
                paths.push(entry.path().to_path_buf());
            }
            paths
        })
    }
}

impl Default for EmbeddedBaseAssetReader {
    fn default() -> Self {
        Self::new()
    }
}

/// Wrapper around static bytes that implements Reader.
pub struct DataReader(pub &'static [u8]);

impl Reader for DataReader {
    fn read_to_end<'a>(
        &'a mut self,
        buf: &'a mut Vec<u8>,
    ) -> bevy::asset::io::StackFuture<
        'a,
        std::io::Result<usize>,
        { bevy::asset::io::STACK_FUTURE_SIZE },
    > {
        let future = futures_lite::AsyncReadExt::read_to_end(self, buf);
        bevy::asset::io::StackFuture::from(future)
    }
}

impl AsyncRead for DataReader {
    fn poll_read(
        self: Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
        buf: &mut [u8],
    ) -> Poll<futures_io::Result<usize>> {
        let read = self.get_mut().0.read(buf);
        Poll::Ready(read)
    }
}

impl AsyncSeek for DataReader {
    fn poll_seek(
        self: Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
        _pos: futures_io::SeekFrom,
    ) -> Poll<futures_io::Result<u64>> {
        Poll::Ready(Err(futures_io::Error::new(
            futures_io::ErrorKind::Unsupported,
            "Seek is not supported for embedded assets",
        )))
    }
}

impl bevy::asset::io::AsyncSeekForward for DataReader {
    fn poll_seek_forward(
        self: Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
        _offset: u64,
    ) -> Poll<futures_io::Result<u64>> {
        Poll::Ready(Err(futures_io::Error::new(
            futures_io::ErrorKind::Unsupported,
            "Seek is not supported for embedded assets",
        )))
    }
}

/// Stream implementation for directory iteration.
struct DirReader(Vec<PathBuf>);

impl Stream for DirReader {
    type Item = PathBuf;

    fn poll_next(
        self: Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        // pop() returns items in reverse order from how they were added,
        // but iteration order is not significant for asset loading
        Poll::Ready(this.0.pop())
    }
}

impl AssetReader for EmbeddedBaseAssetReader {
    async fn read<'a>(&'a self, path: &'a Path) -> Result<impl Reader + 'a, AssetReaderError> {
        self.get_file(path)
            .map(|bytes| {
                let reader: Box<dyn Reader> = Box::new(DataReader(bytes));
                reader
            })
            .ok_or_else(|| AssetReaderError::NotFound(path.to_path_buf()))
    }

    async fn read_meta<'a>(&'a self, path: &'a Path) -> Result<impl Reader + 'a, AssetReaderError> {
        let mut meta_path = path.to_path_buf();
        let mut extension = path
            .extension()
            .map(|e| e.to_os_string())
            .unwrap_or_default();
        extension.push(".meta");
        meta_path.set_extension(extension);

        self.get_file(&meta_path)
            .map(|bytes| {
                let reader: Box<dyn Reader> = Box::new(DataReader(bytes));
                reader
            })
            .ok_or_else(|| AssetReaderError::NotFound(meta_path))
    }

    async fn read_directory<'a>(
        &'a self,
        path: &'a Path,
    ) -> Result<Box<PathStream>, AssetReaderError> {
        self.read_directory(path)
            .map(|paths| {
                let boxed: Box<PathStream> = Box::new(DirReader(paths));
                boxed
            })
            .ok_or_else(|| AssetReaderError::NotFound(path.to_path_buf()))
    }

    async fn is_directory<'a>(&'a self, path: &'a Path) -> Result<bool, AssetReaderError> {
        Ok(self.is_directory(path))
    }
}

/// Create an AssetSourceBuilder that reads from the embedded base assets.
pub fn embedded_asset_source() -> bevy::asset::io::AssetSourceBuilder {
    AssetSource::build().with_reader(|| Box::new(EmbeddedBaseAssetReader::new()))
}
