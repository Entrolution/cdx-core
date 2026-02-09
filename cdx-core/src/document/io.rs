use std::fs::File;
use std::io::{Cursor, Read, Seek, Write};
use std::path::Path;

#[cfg(feature = "encryption")]
use crate::archive::ENCRYPTION_PATH;
#[cfg(feature = "signatures")]
use crate::archive::SIGNATURES_PATH;
use crate::archive::{
    CdxReader, CdxWriter, CompressionMethod, ACADEMIC_NUMBERING_PATH, BIBLIOGRAPHY_PATH,
    COMMENTS_PATH, CONTENT_PATH, DUBLIN_CORE_PATH, FORMS_DATA_PATH, JSONLD_PATH, PHANTOMS_PATH,
};
use crate::content::Content;
use crate::metadata::DublinCore;
use crate::{Hasher, Result};

#[cfg(any(feature = "signatures", feature = "encryption"))]
use crate::manifest::SecurityRef;

use super::Document;

impl Document {
    /// Open a document from a file path.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file cannot be opened
    /// - The archive is invalid
    /// - Required files are missing or malformed
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut reader = CdxReader::open(path)?;
        Self::from_reader(&mut reader)
    }

    /// Open a document from any `Read + Seek` source.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The source is not a valid Codex archive
    /// - Required files are missing or malformed
    pub fn open_from_reader<R: Read + Seek>(reader: R) -> Result<Self> {
        let mut cdx_reader = CdxReader::new(reader)?;
        Self::from_reader(&mut cdx_reader)
    }

    /// Open a document from bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if the data is not a valid Codex document.
    pub fn from_bytes(data: Vec<u8>) -> Result<Self> {
        let mut reader = CdxReader::from_bytes(data)?;
        Self::from_reader(&mut reader)
    }

    /// Read document from a `CdxReader`.
    fn from_reader<R: Read + Seek>(reader: &mut CdxReader<R>) -> Result<Self> {
        let manifest = reader.manifest().clone();

        // Read and parse content
        let content_data = reader.read_content()?;
        let content: Content = serde_json::from_slice(&content_data)?;

        // Read and parse Dublin Core
        let dc_data = reader.read_dublin_core()?;
        let dublin_core: DublinCore = serde_json::from_slice(&dc_data)?;

        // Helper closure to read and parse optional JSON extension files
        let mut read_optional_json = |path: &str| -> Result<Option<Vec<u8>>> {
            if reader.file_exists(path)? {
                Ok(Some(reader.read_file(path)?))
            } else {
                Ok(None)
            }
        };

        // Read signatures if present (only when signatures feature is enabled)
        #[cfg(feature = "signatures")]
        let signature_file = if let Some(ref security) = manifest.security {
            if let Some(ref sig_path) = security.signatures {
                read_optional_json(sig_path)?
                    .map(|data| serde_json::from_slice(&data))
                    .transpose()?
            } else {
                None
            }
        } else {
            None
        };

        // Read encryption metadata if present (only when encryption feature is enabled)
        #[cfg(feature = "encryption")]
        let encryption_metadata = if let Some(ref security) = manifest.security {
            if let Some(ref enc_path) = security.encryption {
                read_optional_json(enc_path)?
                    .map(|data| serde_json::from_slice(&data))
                    .transpose()?
            } else {
                None
            }
        } else {
            None
        };

        // Read extension files using the helper closure
        let academic_numbering = read_optional_json(ACADEMIC_NUMBERING_PATH)?
            .map(|data| serde_json::from_slice(&data))
            .transpose()?;

        let comments = read_optional_json(COMMENTS_PATH)?
            .map(|data| serde_json::from_slice(&data))
            .transpose()?;

        let phantom_clusters = read_optional_json(PHANTOMS_PATH)?
            .map(|data| serde_json::from_slice(&data))
            .transpose()?;

        let form_data = read_optional_json(FORMS_DATA_PATH)?
            .map(|data| serde_json::from_slice(&data))
            .transpose()?;

        let bibliography = read_optional_json(BIBLIOGRAPHY_PATH)?
            .map(|data| serde_json::from_slice(&data))
            .transpose()?;

        let jsonld_metadata = read_optional_json(JSONLD_PATH)?
            .map(|data| serde_json::from_slice(&data))
            .transpose()?;

        Ok(Self {
            manifest,
            content,
            dublin_core,
            #[cfg(feature = "signatures")]
            signature_file,
            #[cfg(feature = "encryption")]
            encryption_metadata,
            academic_numbering,
            comments,
            phantom_clusters,
            form_data,
            bibliography,
            jsonld_metadata,
        })
    }

    /// Save the document to a file.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file cannot be created
    /// - Writing fails
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let file = File::create(path)?;
        let writer = std::io::BufWriter::new(file);
        self.write_to(writer)
    }

    /// Write the document to any `Write + Seek` destination.
    ///
    /// # Errors
    ///
    /// Returns an error if writing fails.
    pub fn write_to<W: Write + Seek>(&self, writer: W) -> Result<()> {
        let mut cdx_writer = CdxWriter::new(writer)?;

        // Serialize content and dublin core
        let content_json = serde_json::to_vec_pretty(&self.content)?;
        let dc_json = serde_json::to_vec_pretty(&self.dublin_core)?;

        // Compute hashes
        let content_hash = Hasher::hash(self.manifest.hash_algorithm, &content_json);

        // Update manifest with computed hashes
        let mut manifest = self.manifest.clone();
        manifest.content.hash = content_hash;

        // Update security reference if we have signatures or encryption
        #[cfg(any(feature = "signatures", feature = "encryption"))]
        {
            #[cfg(feature = "signatures")]
            let has_signatures = self
                .signature_file
                .as_ref()
                .is_some_and(|sf| !sf.is_empty());
            #[cfg(not(feature = "signatures"))]
            let has_signatures = false;

            #[cfg(feature = "encryption")]
            let has_encryption = self.encryption_metadata.is_some();
            #[cfg(not(feature = "encryption"))]
            let has_encryption = false;

            if has_signatures || has_encryption {
                #[cfg(feature = "signatures")]
                let signatures_ref = if has_signatures {
                    Some(SIGNATURES_PATH.to_string())
                } else {
                    None
                };
                #[cfg(not(feature = "signatures"))]
                let signatures_ref = None;

                #[cfg(feature = "encryption")]
                let encryption_ref = if has_encryption {
                    Some(ENCRYPTION_PATH.to_string())
                } else {
                    None
                };
                #[cfg(not(feature = "encryption"))]
                let encryption_ref = None;

                manifest.security = Some(SecurityRef {
                    signatures: signatures_ref,
                    encryption: encryption_ref,
                });
            }
        }

        // Write files
        cdx_writer.write_manifest(&manifest)?;
        cdx_writer.write_file(CONTENT_PATH, &content_json, CompressionMethod::Deflate)?;
        cdx_writer.write_file(DUBLIN_CORE_PATH, &dc_json, CompressionMethod::Deflate)?;

        // Write signatures if present
        #[cfg(feature = "signatures")]
        if let Some(ref sig_file) = self.signature_file {
            if !sig_file.is_empty() {
                let sig_json = sig_file.to_json()?;
                cdx_writer.write_file(
                    SIGNATURES_PATH,
                    sig_json.as_bytes(),
                    CompressionMethod::Deflate,
                )?;
            }
        }

        // Write encryption metadata if present
        #[cfg(feature = "encryption")]
        if let Some(ref enc_meta) = self.encryption_metadata {
            let enc_json = serde_json::to_vec_pretty(enc_meta)?;
            cdx_writer.write_file(ENCRYPTION_PATH, &enc_json, CompressionMethod::Deflate)?;
        }

        // Write optional extension files
        // Using a local macro to avoid repetition while maintaining type safety
        macro_rules! write_optional_json {
            ($path:expr, $value:expr) => {
                if let Some(ref v) = $value {
                    let json = serde_json::to_vec_pretty(v)?;
                    cdx_writer.write_file($path, &json, CompressionMethod::Deflate)?;
                }
            };
        }

        write_optional_json!(ACADEMIC_NUMBERING_PATH, self.academic_numbering);
        write_optional_json!(COMMENTS_PATH, self.comments);
        write_optional_json!(PHANTOMS_PATH, self.phantom_clusters);
        write_optional_json!(FORMS_DATA_PATH, self.form_data);
        write_optional_json!(BIBLIOGRAPHY_PATH, self.bibliography);
        write_optional_json!(JSONLD_PATH, self.jsonld_metadata);

        cdx_writer.finish()?;
        Ok(())
    }

    /// Write the document to bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let cursor = Cursor::new(Vec::new());
        let mut temp = cursor;
        self.write_to(&mut temp)?;
        Ok(temp.into_inner())
    }
}
