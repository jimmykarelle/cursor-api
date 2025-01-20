use super::aiserver::v1::StreamChatResponse;
use flate2::read::GzDecoder;
use prost::Message;
use std::io::Read;

use super::error::{ChatError, StreamError};

// 解压gzip数据
fn decompress_gzip(data: &[u8]) -> Option<Vec<u8>> {
    let mut decoder = GzDecoder::new(data);
    let mut decompressed = Vec::new();

    match decoder.read_to_end(&mut decompressed) {
        Ok(_) => Some(decompressed),
        Err(_) => {
            // println!("gzip解压失败: {}", e);
            None
        }
    }
}

pub enum StreamMessage {
    // 未完成
    Incomplete,
    // 调试
    Debug(String),
    // 流开始标志 b"\0\0\0\0\0"
    StreamStart,
    // 消息内容
    Content(Vec<String>),
    // 流结束标志 b"\x02\0\0\0\x02{}"
    StreamEnd,
}

pub fn parse_stream_data(data: &[u8]) -> Result<StreamMessage, StreamError> {
    if data.len() < 5 {
        return Err(StreamError::DataLengthLessThan5);
    }

    // 检查是否为流开始标志
    // if data == b"\0\0\0\0\0" {
    //     return Ok(StreamMessage::StreamStart);
    // }

    // 检查是否为流结束标志
    // if data == b"\x02\0\0\0\x02{}" {
    //     return Ok(StreamMessage::StreamEnd);
    // }

    let mut messages = Vec::new();
    let mut offset = 0;

    while offset + 5 <= data.len() {
        // 获取消息类型和长度
        let msg_type = data[offset];
        let msg_len = u32::from_be_bytes([
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
            data[offset + 4],
        ]) as usize;

        // 流开始
        if msg_type == 0 && msg_len == 0 {
            return Ok(StreamMessage::StreamStart);
        }

        // 检查剩余数据长度是否足够
        if offset + 5 + msg_len > data.len() {
            return Ok(StreamMessage::Incomplete);
        }

        let msg_data = &data[offset + 5..offset + 5 + msg_len];

        match msg_type {
            // 文本消息
            0 => {
                if let Ok(response) = StreamChatResponse::decode(msg_data) {
                    // crate::debug_println!("[text] StreamChatResponse: {:?}", response);
                    if !response.text.is_empty() {
                        messages.push(response.text);
                    } else {
                        // println!("[text] StreamChatResponse: {:?}", response);
                        return Ok(StreamMessage::Debug(
                            response.filled_prompt.unwrap_or_default(),
                            // response.is_using_slow_request,
                        ));
                    }
                }
            }
            // gzip压缩消息
            1 => {
                if let Some(text) = decompress_gzip(msg_data) {
                    let response = StreamChatResponse::decode(&text[..]).unwrap_or_default();
                    // crate::debug_println!("[gzip] StreamChatResponse: {:?}", response);
                    if !response.text.is_empty() {
                        messages.push(response.text);
                    } else {
                        // println!("[gzip] StreamChatResponse: {:?}", response);
                        return Ok(StreamMessage::Debug(
                            response.filled_prompt.unwrap_or_default(),
                            // response.is_using_slow_request,
                        ));
                    }
                }
            }
            // JSON字符串
            2 => {
                if msg_len == 2 {
                    return Ok(StreamMessage::StreamEnd);
                }
                if let Ok(text) = String::from_utf8(msg_data.to_vec()) {
                    // println!("JSON消息: {}", text);
                    if let Ok(error) = serde_json::from_str::<ChatError>(&text) {
                        return Err(StreamError::ChatError(error));
                    }
                    // 未预计
                    // messages.push(text);
                }
            }
            // 其他类型暂不处理
            t => eprintln!("收到未知消息类型: {}，请尝试联系开发者以获取支持", t),
        }

        offset += 5 + msg_len;
    }

    if messages.is_empty() {
        Err(StreamError::EmptyMessage)
    } else {
        Ok(StreamMessage::Content(messages))
    }
}
