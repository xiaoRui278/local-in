use libp2p::futures::{
    AsyncRead as FuturesAsyncRead, AsyncReadExt as FuturesAsyncReadExt,
    AsyncWrite as FuturesAsyncWrite, AsyncWriteExt as FuturesAsyncWriteExt,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio::fs::File;
use tokio::io::{
    AsyncReadExt as TokioAsyncReadExt, AsyncSeekExt,
    AsyncWriteExt as TokioAsyncWriteExt, SeekFrom,
};
use tokio::sync::{mpsc, watch};

pub const FILE_PROTOCOL: libp2p::StreamProtocol = libp2p::StreamProtocol::new("/local-in-file/1");
pub const DEFAULT_CHUNK_SIZE: u64 = 1024 * 1024;
pub const ACK_BYTES_INTERVAL: u64 = 16 * 1024 * 1024;
pub const ACK_MILLIS_INTERVAL: u64 = 500;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FrameType {
    Init = 1,
    Data = 2,
    Ack = 3,
    Complete = 4,
    Cancel = 5,
    Error = 6,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TransferInit {
    pub file_id: String,
    pub filename: String,
    pub file_size: u64,
    pub chunk_size: u64,
    pub sha256: String,
    pub resume_offset: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TransferAck {
    pub file_id: String,
    pub received_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TransferComplete {
    pub file_id: String,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TransferCancel {
    pub file_id: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TransferError {
    pub file_id: String,
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct OutgoingFile {
    pub file_id: String,
    pub filename: String,
    pub path: PathBuf,
    pub file_size: u64,
    pub sha256: String,
    pub resume_offset: u64,
}

#[derive(Debug, Clone)]
pub struct IncomingFileTarget {
    pub file_id: String,
    pub from_peer: String,
    pub resume_offset: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransferPhase {
    Hashing,
    Transferring,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FileTransferEvent {
    Progress {
        file_id: String,
        status: String,
        phase: TransferPhase,
        received_size: u64,
        total_size: u64,
        speed: u64,
    },
    Completed {
        file_id: String,
        file_path: String,
    },
    Failed {
        file_id: String,
        error_message: String,
    },
    Cancelled {
        file_id: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransferFrame {
    pub frame_type: FrameType,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataPayload {
    pub offset: u64,
    pub bytes: Vec<u8>,
}

impl TryFrom<u8> for FrameType {
    type Error = io::Error;

    fn try_from(value: u8) -> Result<Self, <FrameType as TryFrom<u8>>::Error> {
        match value {
            1 => Ok(Self::Init),
            2 => Ok(Self::Data),
            3 => Ok(Self::Ack),
            4 => Ok(Self::Complete),
            5 => Ok(Self::Cancel),
            6 => Ok(Self::Error),
            _ => Err(io::Error::new(io::ErrorKind::InvalidData, "unknown frame type")),
        }
    }
}

pub async fn write_frame<W>(writer: &mut W, frame_type: FrameType, payload: &[u8]) -> io::Result<()>
where
    W: FuturesAsyncWrite + Unpin,
{
    let mut header = [0u8; 9];
    header[0] = frame_type as u8;
    header[1..].copy_from_slice(&(payload.len() as u64).to_be_bytes());
    writer.write_all(&header).await?;
    writer.write_all(payload).await?;
    writer.flush().await?;
    Ok(())
}

pub async fn read_frame<R>(reader: &mut R) -> io::Result<Option<TransferFrame>>
where
    R: FuturesAsyncRead + Unpin,
{
    let mut header = [0u8; 9];
    match reader.read_exact(&mut header).await {
        Ok(_) => {}
        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e),
    }
    let frame_type = FrameType::try_from(header[0])?;
    let mut payload_len_bytes = [0u8; 8];
    payload_len_bytes.copy_from_slice(&header[1..]);
    let payload_len = u64::from_be_bytes(payload_len_bytes);
    if payload_len > 64 * 1024 * 1024 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "frame too large"));
    }

    let mut payload = vec![0; payload_len as usize];
    reader.read_exact(&mut payload).await?;
    Ok(Some(TransferFrame { frame_type, payload }))
}

pub fn encode_data_payload(offset: u64, bytes: &[u8]) -> Vec<u8> {
    let mut payload = Vec::with_capacity(12 + bytes.len());
    payload.extend_from_slice(&offset.to_be_bytes());
    payload.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
    payload.extend_from_slice(bytes);
    payload
}

pub fn decode_data_payload(payload: &[u8]) -> io::Result<DataPayload> {
    if payload.len() < 12 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "data payload too short"));
    }
    let mut offset = [0; 8];
    offset.copy_from_slice(&payload[..8]);
    let mut len = [0; 4];
    len.copy_from_slice(&payload[8..12]);
    let length = u32::from_be_bytes(len) as usize;
    if payload.len() != 12 + length {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "data payload length mismatch"));
    }
    Ok(DataPayload {
        offset: u64::from_be_bytes(offset),
        bytes: payload[12..].to_vec(),
    })
}

pub fn encode_json_payload<T: Serialize>(value: &T) -> io::Result<Vec<u8>> {
    serde_json::to_vec(value).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

pub fn decode_json_payload<T>(payload: &[u8]) -> io::Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    serde_json::from_slice(payload).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

pub fn trusted_resume_offset(db_received_bytes: u64, temp_file_len: u64) -> u64 {
    db_received_bytes.min(temp_file_len)
}

pub fn calculate_speed(bytes_delta: u64, elapsed_seconds: u64) -> u64 {
    if elapsed_seconds == 0 {
        return 0;
    }
    bytes_delta / elapsed_seconds
}

/// Independent full-file SHA256, used as a test oracle for `hash_file_range`.
#[cfg(test)]
pub async fn sha256_file(path: &std::path::Path) -> io::Result<String> {
    let mut file = File::open(path).await?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0; DEFAULT_CHUNK_SIZE as usize];
    loop {
        let read = file.read(&mut buffer).await?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hex::encode(hasher.finalize()))
}

/// Reads `[0, len)` bytes from `path`, feeding them into a fresh SHA256 hasher,
/// and returns the (unfinalized) hasher so the caller can continue updating it.
/// Passing `u64::MAX` hashes the entire file. Used to seed the incremental
/// hash when resuming a transfer from an offset.
pub async fn hash_file_range(path: &std::path::Path, len: u64) -> io::Result<Sha256> {
    let mut file = File::open(path).await?;
    let mut hasher = Sha256::new();
    let mut remaining = len;
    let mut buffer = vec![0; DEFAULT_CHUNK_SIZE as usize];
    while remaining > 0 {
        let want = remaining.min(buffer.len() as u64) as usize;
        let read = file.read(&mut buffer[..want]).await?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
        remaining -= read as u64;
    }
    Ok(hasher)
}

pub fn validate_data_offset(actual: u64, expected: u64) -> io::Result<()> {
    if actual != expected {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "unexpected data offset"));
    }
    Ok(())
}

pub async fn send_file_stream(
    mut stream: libp2p::Stream,
    file: OutgoingFile,
    cancel_rx: watch::Receiver<bool>,
    events: mpsc::Sender<FileTransferEvent>,
) -> io::Result<()> {
    let init = TransferInit {
        file_id: file.file_id.clone(),
        filename: file.filename.clone(),
        file_size: file.file_size,
        chunk_size: DEFAULT_CHUNK_SIZE,
        sha256: file.sha256.clone(),
        resume_offset: file.resume_offset,
    };
    write_frame(&mut stream, FrameType::Init, &encode_json_payload(&init)?).await?;

    let mut source = File::open(&file.path).await?;
    source.seek(SeekFrom::Start(file.resume_offset)).await?;

    // Seed the incremental hash with the bytes already sent (0 when not resuming),
    // so the trailing hash covers the whole file regardless of resume offset.
    let mut hasher = hash_file_range(&file.path, file.resume_offset).await?;

    let mut offset = file.resume_offset;
    let mut buffer = vec![0; DEFAULT_CHUNK_SIZE as usize];
    let started = Instant::now();
    let mut last_event_at = Instant::now();
    let mut last_event_bytes = offset;

    loop {
        if *cancel_rx.borrow() {
            let cancel = TransferCancel {
                file_id: file.file_id.clone(),
                reason: "user_cancelled".to_string(),
            };
            let _ = write_frame(&mut stream, FrameType::Cancel, &encode_json_payload(&cancel)?).await;
            let _ = events
                .send(FileTransferEvent::Cancelled {
                    file_id: file.file_id.clone(),
                })
                .await;
            return Ok(());
        }

        let read = source.read(&mut buffer).await?;
        if read == 0 {
            break;
        }

        hasher.update(&buffer[..read]);
        let payload = encode_data_payload(offset, &buffer[..read]);
        write_frame(&mut stream, FrameType::Data, &payload).await?;
        offset += read as u64;

        if last_event_at.elapsed() >= Duration::from_millis(ACK_MILLIS_INTERVAL) || offset == file.file_size {
            let elapsed = last_event_at.elapsed().as_secs().max(1);
            let speed = calculate_speed(offset - last_event_bytes, elapsed);
            let _ = events
                .send(FileTransferEvent::Progress {
                    file_id: file.file_id.clone(),
                    status: "transferring".to_string(),
                    phase: TransferPhase::Transferring,
                    received_size: offset,
                    total_size: file.file_size,
                    speed,
                })
                .await;
            last_event_at = Instant::now();
            last_event_bytes = offset;
        }
    }

    let ack = TransferAck {
        file_id: file.file_id.clone(),
        received_bytes: offset,
    };
    write_frame(&mut stream, FrameType::Ack, &encode_json_payload(&ack)?).await?;

    let complete = TransferComplete {
        file_id: file.file_id.clone(),
        sha256: hex::encode(hasher.finalize()),
    };
    write_frame(&mut stream, FrameType::Complete, &encode_json_payload(&complete)?).await?;
    stream.close().await?;

    let total_elapsed = started.elapsed().as_secs().max(1);
    let _ = events
        .send(FileTransferEvent::Progress {
            file_id: file.file_id,
            status: "completed".to_string(),
            phase: TransferPhase::Transferring,
            received_size: file.file_size,
            total_size: file.file_size,
            speed: calculate_speed(file.file_size.saturating_sub(file.resume_offset), total_elapsed),
        })
        .await;

    Ok(())
}

pub async fn receive_file_stream(
    _peer_id: String,
    mut stream: libp2p::Stream,
    events: mpsc::Sender<FileTransferEvent>,
) -> io::Result<()> {
    let init_frame = read_frame(&mut stream)
        .await?
        .ok_or_else(|| io::Error::new(io::ErrorKind::UnexpectedEof, "missing init frame"))?;
    if init_frame.frame_type != FrameType::Init {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "first frame must be init"));
    }
    let init: TransferInit = decode_json_payload(&init_frame.payload)?;

    let download_dir = dirs::download_dir().unwrap_or_default();
    let temp_path = download_dir.join(format!("{}.localin.part", init.filename));
    let final_path = download_dir.join(&init.filename);

    let mut output = tokio::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .read(true)
        .open(&temp_path)
        .await?;
    output.set_len(init.resume_offset).await?;
    output.seek(SeekFrom::Start(init.resume_offset)).await?;

    // Seed the incremental hash with the bytes already on disk (0 when not
    // resuming), so verification covers the whole file at completion.
    let mut hasher = hash_file_range(&temp_path, init.resume_offset).await?;

    let mut expected_offset = init.resume_offset;
    let mut last_event_at = Instant::now();
    let mut last_event_bytes = expected_offset;

    loop {
        let Some(frame) = read_frame(&mut stream).await? else { break; };
        match frame.frame_type {
            FrameType::Data => {
                let data = decode_data_payload(&frame.payload)?;
                validate_data_offset(data.offset, expected_offset)?;
                output.write_all(&data.bytes).await?;
                hasher.update(&data.bytes);
                expected_offset += data.bytes.len() as u64;

                if expected_offset.saturating_sub(last_event_bytes) >= ACK_BYTES_INTERVAL
                    || last_event_at.elapsed() >= Duration::from_millis(ACK_MILLIS_INTERVAL)
                    || expected_offset == init.file_size
                {
                    let ack = TransferAck {
                        file_id: init.file_id.clone(),
                        received_bytes: expected_offset,
                    };
                    write_frame(&mut stream, FrameType::Ack, &encode_json_payload(&ack)?).await?;

                    let elapsed = last_event_at.elapsed().as_secs().max(1);
                    let speed = calculate_speed(expected_offset - last_event_bytes, elapsed);
                    let _ = events
                        .send(FileTransferEvent::Progress {
                            file_id: init.file_id.clone(),
                            status: "transferring".to_string(),
                            phase: TransferPhase::Transferring,
                            received_size: expected_offset,
                            total_size: init.file_size,
                            speed,
                        })
                        .await;
                    last_event_at = Instant::now();
                    last_event_bytes = expected_offset;
                }
            }
            FrameType::Complete => {
                output.flush().await?;
                drop(output);
                let complete: TransferComplete = decode_json_payload(&frame.payload)?;
                let local_hash = hex::encode(hasher.finalize());
                if local_hash != complete.sha256 {
                    let message = "Received file hash did not match sender hash".to_string();
                    let _ = events
                        .send(FileTransferEvent::Failed {
                            file_id: init.file_id.clone(),
                            error_message: message.clone(),
                        })
                        .await;
                    return Err(io::Error::new(io::ErrorKind::InvalidData, message));
                }
                tokio::fs::rename(&temp_path, &final_path).await?;
                let _ = events
                    .send(FileTransferEvent::Completed {
                        file_id: init.file_id.clone(),
                        file_path: final_path.to_string_lossy().to_string(),
                    })
                    .await;
                return Ok(());
            }
            FrameType::Cancel => {
                let _ = events
                    .send(FileTransferEvent::Cancelled {
                        file_id: init.file_id.clone(),
                    })
                    .await;
                return Ok(());
            }
            FrameType::Error => {
                let error: TransferError = decode_json_payload(&frame.payload)?;
                let _ = events
                    .send(FileTransferEvent::Failed {
                        file_id: init.file_id.clone(),
                        error_message: error.message.clone(),
                    })
                    .await;
                return Err(io::Error::new(io::ErrorKind::Other, error.message));
            }
            FrameType::Ack | FrameType::Init => {}
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn frame_round_trip_preserves_type_and_payload() {
        let payload = b"hello-binary".to_vec();
        let mut stream = futures::io::Cursor::new(Vec::new());

        write_frame(&mut stream, FrameType::Data, &payload).await.unwrap();
        stream.set_position(0);

        let frame = read_frame(&mut stream).await.unwrap().unwrap();
        assert_eq!(frame.frame_type, FrameType::Data);
        assert_eq!(frame.payload, b"hello-binary");
    }

    #[tokio::test]
    async fn data_payload_round_trip_preserves_offset_and_bytes() {
        let payload = encode_data_payload(4096, b"abc");
        let decoded = decode_data_payload(&payload).unwrap();
        assert_eq!(decoded.offset, 4096);
        assert_eq!(decoded.bytes, b"abc");
    }

    #[tokio::test]
    async fn json_payload_round_trip_preserves_init() {
        let init = TransferInit {
            file_id: "file-1".to_string(),
            filename: "a.bin".to_string(),
            file_size: 9,
            chunk_size: DEFAULT_CHUNK_SIZE,
            sha256: "hash".to_string(),
            resume_offset: 3,
        };
        let payload = encode_json_payload(&init).unwrap();
        let decoded: TransferInit = decode_json_payload(&payload).unwrap();
        assert_eq!(decoded, init);
    }

    #[test]
    fn trusted_resume_offset_uses_smaller_db_and_file_length() {
        assert_eq!(trusted_resume_offset(100, 80), 80);
        assert_eq!(trusted_resume_offset(80, 100), 80);
        assert_eq!(trusted_resume_offset(64, 64), 64);
    }

    #[test]
    fn data_offset_must_match_expected_offset() {
        assert!(validate_data_offset(1024, 1024).is_ok());
        assert!(validate_data_offset(2048, 1024).is_err());
    }

    #[tokio::test]
    async fn hash_file_range_over_full_length_matches_sha256_file() {
        let path = std::env::temp_dir().join(format!(
            "localin-hashrange-{}.bin",
            uuid::Uuid::new_v4()
        ));
        let data: Vec<u8> = (0..100_000u32).map(|i| (i % 251) as u8).collect();
        tokio::fs::write(&path, &data).await.unwrap();

        let full = sha256_file(&path).await.unwrap();

        // Seeding over the full length then finalizing equals the full-file hash.
        let hasher = hash_file_range(&path, data.len() as u64).await.unwrap();
        assert_eq!(hex::encode(hasher.finalize()), full);

        // Seeding a prefix then continuing with the remaining bytes equals the full-file hash.
        let mut partial = hash_file_range(&path, 40_000).await.unwrap();
        partial.update(&data[40_000..]);
        assert_eq!(hex::encode(partial.finalize()), full);

        tokio::fs::remove_file(&path).await.ok();
    }
}
