use dcmfx::core::*;
use dcmfx::p10::*;
use dcmfx::p10::p10_write::{data_set_to_bytes, P10WriteConfig};
use wasm_bindgen::prelude::*;
use base64::{engine::general_purpose::STANDARD, Engine};
use dcmfx::pixel_data::DataSetPixelDataExtensions;
use image::{codecs::png::PngEncoder, imageops};
use image::ImageEncoder;
use serde::Serialize;
use wasm_bindgen::JsValue;
use std::rc::Rc;
use serde_json;
use dcmfx::core::dictionary::tag_name;

#[derive(Serialize)]
struct DicomResult {
    image_data: Vec<String>,  // 每个元素是一帧的base64编码PNG图像
    metadata: Vec<TagInfo>,
    number_of_frames: i32,
}

#[derive(Serialize)]
struct TagInfo {
    tag: String,
    name: String,
    value: String,
}

#[wasm_bindgen]
pub fn read_dicom(file_data: &[u8], quality: &str) -> String {
    let result: Result<DicomResult, String> = (|| {
        if file_data.len() < 132 {
            return Err("文件太小，不是有效的DICOM文件".to_string());
        }

        web_sys::console::log_1(&"开始读取DICOM文件".into());
        
        let preamble = &file_data[128..132];
        if preamble != b"DICM" {
            return Err("不是有效的DICOM文件格式".to_string());
        }

        let mut stream = &file_data[..];
        let ds = match DataSet::read_p10_stream(&mut stream) {
            Ok(ds) => {
                web_sys::console::log_1(&"成功读取DICOM数据集".into());
                ds
            },
            Err(e) => {
                web_sys::console::error_1(&format!("DICOM解析错误: {}", e).into());
                return Err(format!("读取DICOM文件失败: {}", e));
            }
        };

        match (ds.get_int(dictionary::COLUMNS.tag), ds.get_int(dictionary::ROWS.tag)) {
            (Ok(_), Ok(_)) => {
                // 继续处理
            },
            _ => {
                return Err("缺少必要的图像尺寸信息".to_string());
            }
        }

        let (width, height) = {
            let width = ds.get_int(dictionary::COLUMNS.tag)
                .map_err(|e| {
                    web_sys::console::error_1(&format!("获取宽度失败: {}", e).into());
                    "无法获取图像宽度"
                })?;
            let height = ds.get_int(dictionary::ROWS.tag)
                .map_err(|e| {
                    web_sys::console::error_1(&format!("获取高度失败: {}", e).into());
                    "无法获取图像高度"
                })?;
            
            web_sys::console::log_1(&format!("图像尺寸: {}x{}", width, height).into());
            
            if width <= 0 || height <= 0 {
                return Err("图像尺寸无效".to_string());
            }
            
            (width as usize, height as usize)
        };

        let (transfer_syntax, pixel_data) = match ds.get_pixel_data() {
            Ok(data) => {
                web_sys::console::log_1(&"成功获取像素数据".into());
                data
            },
            Err(e) => {
                web_sys::console::error_1(&format!("获取像素数据失败: {}", e).into());
                return Err("无法提取像素数据".to_string());
            }
        };

        web_sys::console::log_1(&format!("传输语法: {:?}", transfer_syntax).into());

        let flat_pixel_data: Vec<u8> = pixel_data
            .into_iter()
            .flat_map(|v| v.into_iter())
            .flat_map(|slice| slice.iter().copied())
            .collect();

        web_sys::console::log_1(&format!("像素数据大小: {}", flat_pixel_data.len()).into());

        let samples_per_pixel = ds.get_int(dictionary::SAMPLES_PER_PIXEL.tag)
            .map_err(|e| format!("无法获取 Samples per Pixel: {}", e))?;

        let photometric_interpretation = ds.get_string(dictionary::PHOTOMETRIC_INTERPRETATION.tag)
            .map_err(|e| format!("无法获取 Photometric Interpretation: {}", e))?;

        let bits_allocated = ds.get_int(dictionary::BITS_ALLOCATED.tag)
            .map_err(|e| format!("无法获取 Bits Allocated: {}", e))?;

        let bits_stored = ds.get_int(dictionary::BITS_STORED.tag)
            .map_err(|e| format!("无法获取 Bits Stored: {}", e))?;

        let high_bit = ds.get_int(dictionary::HIGH_BIT.tag)
            .map_err(|e| format!("无法获取 High Bit: {}", e))?;

        let pixel_representation = ds.get_int(dictionary::PIXEL_REPRESENTATION.tag)
            .map_err(|e| format!("无法获取 Pixel Representation: {}", e))?;

        web_sys::console::log_1(&format!("Bits Allocated: {}, Samples per Pixel: {}, Photometric Interpretation: {}, Bits Stored: {}, High Bit: {}, Pixel Representation: {}", 
            bits_allocated, samples_per_pixel, photometric_interpretation, bits_stored, high_bit, pixel_representation).into());

        let number_of_frames = ds.get_int(dictionary::NUMBER_OF_FRAMES.tag).unwrap_or(1);
        let samples_per_pixel = ds.get_int(dictionary::SAMPLES_PER_PIXEL.tag)
            .map_err(|e| format!("无法获取 Samples per Pixel: {}", e))?;
        let planar_configuration = ds.get_int(dictionary::PLANAR_CONFIGURATION.tag).unwrap_or(0);
        let rows = height as u32;
        let columns = width as u32;

        let expected_frame_size = match bits_allocated {
            8 => rows * columns * samples_per_pixel as u32,
            16 => rows * columns * samples_per_pixel as u32 * 2,
            _ => return Err(format!("不支持的位深度: {}", bits_allocated))
        };

        let expected_total_size = expected_frame_size * number_of_frames as u32;

        web_sys::console::log_1(&format!(
            "详细信息:\n帧数: {}\n每像素样: {}\n平面配置: {}\n行数: {}\n列数: {}\n预期每帧大小: {}\n预期总大小: {}\n实际大小: {}", 
            number_of_frames,
            samples_per_pixel,
            planar_configuration,
            rows,
            columns,
            expected_frame_size,
            expected_total_size,
            flat_pixel_data.len()
        ).into());

        if flat_pixel_data.len() as u32 != expected_total_size {
            return Err(format!(
                "像素数据大小不匹配: 预期 {} 字节 ({}x{}x{}x{}x{}), 实际 {} 字节",
                expected_total_size,
                rows,
                columns,
                samples_per_pixel,
                if bits_allocated == 16 { 2 } else { 1 },
                number_of_frames,
                flat_pixel_data.len()
            ));
        }

        let frame_size = (expected_frame_size) as usize;

        // 在处理像素数据时添加降采样逻辑
        let scale_factor = match quality {
            "high" => 2,
            "medium" => 4,
            "low" => 8,
            _ => 1, // "full" 或其他情况使用原始尺寸
        };

        // 检查压缩相关标签
        let lossy_image_compression = ds.get_string(dictionary::LOSSY_IMAGE_COMPRESSION.tag).ok();
        let lossy_compression_ratio = ds.get_string(dictionary::LOSSY_IMAGE_COMPRESSION_RATIO.tag).ok();
        let lossy_compression_method = ds.get_string(dictionary::LOSSY_IMAGE_COMPRESSION_METHOD.tag).ok();

        if let Some(compression) = lossy_image_compression {
            web_sys::console::log_1(&format!("有损压缩: {}", compression).into());
            if let Some(ratio) = lossy_compression_ratio {
                web_sys::console::log_1(&format!("压缩比: {}", ratio).into());
            }
            if let Some(method) = lossy_compression_method {
                web_sys::console::log_1(&format!("压缩方法: {}", method).into());
            }
        }

        // 处理所有帧
        let mut frame_images = Vec::new();
        for frame_index in 0..number_of_frames {
            let frame_start = frame_index as usize * frame_size;
            let frame_end = frame_start + frame_size;
            let frame_data = &flat_pixel_data[frame_start..frame_end];

            web_sys::console::log_1(&format!("处理第{}帧，帧大小: {} 字节", frame_index + 1, frame_size).into());

            let processed_pixel_data: Vec<u8> = if photometric_interpretation == "MONOCHROME1" || photometric_interpretation == "MONOCHROME2" {
                // 处理灰度图像
                if bits_allocated == 16 {
                    web_sys::console::log_1(&format!(
                        "处理16位灰度图像: Photometric={}, BitsStored={}, HighBit={}, PixelRepresentation={}",
                        photometric_interpretation, bits_stored, high_bit, pixel_representation
                    ).into());

                    let mut sample_count = 0;
                    let mut min_value = u16::MAX;
                    let mut max_value = 0u16;

                    // 首先扫描找出实际的值范围
                    for chunk in frame_data.chunks(2) {
                        if chunk.len() == 2 {
                            let raw = ((chunk[1] as u16) << 8) | (chunk[0] as u16);
                            if raw > 0 {
                                min_value = min_value.min(raw);
                                max_value = max_value.max(raw);
                            }
                        }
                    }

                    web_sys::console::log_1(&format!(
                        "像素值范围: min={}, max={}", 
                        min_value, max_value
                    ).into());

                    frame_data.chunks(2)
                        .map(|chunk| {
                            if chunk.len() == 2 {
                                let raw = ((chunk[1] as u16) << 8) | (chunk[0] as u16);
                                
                                // 记录本值
                                if frame_index == 0 && raw > 0 && sample_count < 5 {
                                    web_sys::console::log_1(&format!(
                                        "样本值 {}: raw={}", 
                                        sample_count + 1,
                                        raw
                                    ).into());
                                    sample_count += 1;
                                }

                                // 如果最大值和最小值相同，返回中间值
                                if max_value == min_value {
                                    return 128;
                                }

                                // 根据实
                                let normalized = if raw <= min_value {
                                    0
                                } else if raw >= max_value {
                                    255
                                } else {
                                    ((raw - min_value) as f32 / (max_value - min_value) as f32 * 255.0) as u8
                                };

                                // 根据光度解释进行反转
                                if photometric_interpretation == "MONOCHROME1" {
                                    255 - normalized
                                } else {
                                    normalized
                                }
                            } else {
                                web_sys::console::error_1(&"存在不匹配的像素数据".into());
                                0
                            }
                        })
                        .collect()
                } else {
                    web_sys::console::log_1(&format!(
                        "处理{}位灰度图像: Photometric={}", 
                        bits_allocated, photometric_interpretation
                    ).into());

                    if photometric_interpretation == "MONOCHROME1" {
                        frame_data.iter().map(|&v| 255 - v).collect()
                    } else {
                        frame_data.to_vec()
                    }
                }
            } else {
                web_sys::console::log_1(&format!(
                    "处理其他类型图像: Photometric={}", 
                    photometric_interpretation
                ).into());
                frame_data.to_vec()
            };

            // 检查处理后的数据
            if frame_index == 0 {
                let black_pixels = processed_pixel_data.iter().filter(|&&x| x == 0).count();
                let white_pixels = processed_pixel_data.iter().filter(|&&x| x == 255).count();
                web_sys::console::log_1(&format!(
                    "像素统计: 总数={}, 黑色={}, 白色={}", 
                    processed_pixel_data.len(),
                    black_pixels,
                    white_pixels
                ).into());
            }

            // 创建图像缓冲区后，根据quality进行质量调整
            let image_buffer = match image::GrayImage::from_raw(
                width as u32,
                height as u32,
                processed_pixel_data
            ) {
                Some(buffer) => {
                    if scale_factor > 1 {
                        // 使用Lanczos插值进行降采样和上采样，保持原始尺寸
                        let filtered_image = imageops::blur(&buffer, match quality {
                            "high" => 1.2,     // 轻微模糊
                            "medium" => 2.0,   // 中等模糊
                            "low" => 3.0,      // 较强模糊
                            _ => 0.0,         // 原始质量
                        });
                        filtered_image
                    } else {
                        buffer
                    }
                },
                None => {
                    web_sys::console::error_1(&format!(
                        "创建第{}帧图像缓冲区失败", frame_index + 1
                    ).into());
                    continue;
                }
            };

            // 编码为PNG
            let mut png_data = Vec::new();
            let encoder = PngEncoder::new(&mut png_data);
            if let Err(e) = encoder.write_image(
                &image_buffer.as_raw(),
                image_buffer.width(),
                image_buffer.height(),
                image::ColorType::L8.into()
            ) {
                web_sys::console::error_1(&format!("第{}帧PNG编码失败: {}", frame_index + 1, e).into());
                continue;
            }

            // 将PNG数据转换为base64并存储
            frame_images.push(STANDARD.encode(&png_data));
            web_sys::console::log_1(&format!(
                "第{}帧处理完成 ({}x{})", 
                frame_index + 1,
                image_buffer.width(),
                image_buffer.height()
            ).into());

            // 只在处理完成时打印简短的状态
            web_sys::console::log_1(&format!("帧 {}/{} 处理完成", frame_index + 1, number_of_frames).into());
        }

        // 集标签信息
        let mut metadata = Vec::new();
        for tag in ds.tags() {
            if let Some(tag_info) = extract_tag_info(&ds, tag) {
                metadata.push(tag_info);
            }
        }

        Ok(DicomResult {
            image_data: frame_images,
            metadata,
            number_of_frames: number_of_frames as i32,
        })
    })();

    match result {
        Ok(dicom_result) => {
            web_sys::console::log_1(&"开始序列化结果".into());
            web_sys::console::log_1(&format!("图像数据长度: {}", dicom_result.image_data.len()).into());
            web_sys::console::log_1(&format!("标签数据条目数: {}", dicom_result.metadata.len()).into());

            match serde_json::to_string(&dicom_result) {
                Ok(json_string) => {
                    web_sys::console::log_1(&"JSON 序列化成功".into());
                    json_string
                },
                Err(e) => {
                    web_sys::console::error_1(&format!("JSON 序列化失败: {}", e).into());
                    format!("序列化错误: {}", e)
                }
            }
        },
        Err(e) => {
            web_sys::console::error_1(&format!("处理 DICOM 文件时发生错误: {}", e).into());
            format!("Error: {}", e)
        }
    }
}

fn extract_tag_info(ds: &DataSet, tag: DataElementTag) -> Option<TagInfo> {
    let name = tag_name(tag, None); // 使用标签字典获取名称

    let value = match ds.get_string(tag) {
        Ok(s) => s.to_string(),
        Err(_) => match ds.get_int(tag) {
            Ok(i) => i.to_string(),
            Err(_) => match ds.get_float(tag) {
                Ok(f) => f.to_string(),
                Err(_) => return None, // 无法读取值时返回None
            },
        },
    };

    Some(TagInfo {
        tag: format!("({:04X},{:04X})", tag.group, tag.element),
        name: name.to_string(),
        value,
    })
}


// 流式压缩函数
fn compress_dicom_stream(input_data: &[u8], quality: u32) -> Result<Vec<u8>, P10Error> {
    // 1. 读取原始数据集
    let mut stream = &input_data[..];
    let input_dataset = DataSet::read_p10_stream(&mut stream)?;
    
    // 2. 创建新的压缩格式数据集
    let mut output_dataset = DataSet::new();
    
    // 3. 设置必要的传输语法
    let uid = if quality >= 100 {
        "1.2.840.10008.1.2.4.90" // JPEG 2000 Lossless
    } else {
        "1.2.840.10008.1.2.4.91" // JPEG 2000 Lossy
    };
    let mut bytes = format!("{}\0", uid).into_bytes();
    if bytes.len() % 2 != 0 {
        bytes.push(0);
    }
    output_dataset.insert(
        dictionary::TRANSFER_SYNTAX_UID.tag,
        DataElementValue::new_binary(
            ValueRepresentation::UniqueIdentifier,
            std::rc::Rc::new(bytes)
        ).unwrap()
    );

    // 4. 复制必要的标签 (与原函数相同)
    let required_tags = [
        dictionary::PATIENT_ID.tag,
        dictionary::PATIENT_NAME.tag,
        dictionary::STUDY_INSTANCE_UID.tag,
        dictionary::SERIES_INSTANCE_UID.tag,
        dictionary::SOP_INSTANCE_UID.tag,
        dictionary::ROWS.tag,
        dictionary::COLUMNS.tag,
        dictionary::BITS_ALLOCATED.tag,
        dictionary::BITS_STORED.tag,
        dictionary::HIGH_BIT.tag,
        dictionary::PIXEL_REPRESENTATION.tag,
        dictionary::SAMPLES_PER_PIXEL.tag,
        dictionary::PHOTOMETRIC_INTERPRETATION.tag,
        dictionary::WINDOW_CENTER.tag,
        dictionary::WINDOW_WIDTH.tag,
        dictionary::PIXEL_SPACING.tag,
        dictionary::IMAGE_ORIENTATION_PATIENT.tag,
        dictionary::IMAGE_POSITION_PATIENT.tag,
    ];

    for tag in required_tags.iter() {
        if let Ok(value) = input_dataset.get_value(*tag) {
            output_dataset.insert(*tag, value.clone());
        }
    }

    // 5. 获取并处理像素数据
    if let Ok(pixel_data) = input_dataset.get_value(dictionary::PIXEL_DATA.tag) {
        if let Ok(items) = pixel_data.encapsulated_pixel_data() {
            // 如果已经是压缩格式，直接复制
            output_dataset.insert(dictionary::PIXEL_DATA.tag, pixel_data.clone());
        } else if let Ok(bytes) = pixel_data.bytes() {
            // 如果是原始格式，需要进行压缩
            let rows = match input_dataset.get_int(dictionary::ROWS.tag) {
                Ok(v) => v as usize,
                Err(_) => return Err(P10Error::DataInvalid {
                    when: "Reading rows".to_string(),
                    details: "Missing or invalid rows".to_string(),
                    path: DataSetPath::new(),
                    offset: 0,
                }),
            };
            let columns = match input_dataset.get_int(dictionary::COLUMNS.tag) {
                Ok(v) => v as usize,
                Err(_) => return Err(P10Error::DataInvalid {
                    when: "Reading columns".to_string(),
                    details: "Missing or invalid columns".to_string(),
                    path: DataSetPath::new(),
                    offset: 0,
                }),
            };

            // Bits Allocated = 16
            // Bits Stored = 12
            // High Bit = 11
            // 解释：

            // 每个像素分配了 16 位存储空间。
            // 实际存储有效数据只用了 12 位。
            // 有效数据的最高位是第 11 位（从 0 开始计算）。
            let bits_allocated = match input_dataset.get_int(dictionary::BITS_ALLOCATED.tag) {
                Ok(v) => v as usize,
                Err(_) => return Err(P10Error::DataInvalid {
                    when: "Reading bits allocated".to_string(),
                    details: "Missing or invalid bits allocated".to_string(),
                    path: DataSetPath::new(),
                    offset: 0,
                }),
            };
            let bits_stored = match input_dataset.get_int(dictionary::BITS_STORED.tag) {
                Ok(v) => v as usize,
                Err(_) => bits_allocated, // 默认等于 bits_allocated
            };
            let high_bit = match input_dataset.get_int(dictionary::HIGH_BIT.tag) {
                Ok(v) => v as usize,
                Err(_) => bits_stored - 1, // 默认等于 bits_stored - 1
            };
            let samples_per_pixel = match input_dataset.get_int(dictionary::SAMPLES_PER_PIXEL.tag) {
                Ok(v) => v as usize,
                Err(_) => 1, // 默认值
            };
            let number_of_frames = match input_dataset.get_int(dictionary::NUMBER_OF_FRAMES.tag) {
                Ok(v) => v as usize,
                Err(_) => 1, // 默认值
            };

            // 从总大小反推每帧的大小
            let total_size = bytes.len();
            let frame_size = total_size / number_of_frames;
            //8byte = 1字节 计算字节数
            let bytes_per_pixel = bits_allocated / 8;

            // 根据质量参数确定采样步长
            let step = if bits_allocated == 16 {
                if quality >= 90 {
                    1  // 原图质量 (100%)
                } else if quality >= 75 {
                    2  // 高质量 (50%)
                } else if quality >= 50 {
                    4  // 中等质量 (25%)
                } else if quality >= 25 {
                    8  // 低质量 (12.5%)
                } else {
                    16  // 最低质量 (6.25%)
                }
            } else {
                1  // 非16位图像不进行降采样
            };

            if step > 1 {
                let new_rows = rows / step;
                let new_columns = columns / step;
                let new_frame_size = new_rows * new_columns * bytes_per_pixel * samples_per_pixel;
                
                // 创建新的像素数据
                let mut new_bytes = Vec::with_capacity(new_frame_size * number_of_frames);
                
                // 对每一帧进行处理
                for frame in 0..number_of_frames {
                    let src_frame_start = frame * frame_size;
                    let src_frame_end = src_frame_start + frame_size;
                    let frame_bytes = &bytes[src_frame_start..src_frame_end];
                    
                    // 处理当前帧
                    for y in (0..rows).step_by(step) {
                        for x in (0..columns).step_by(step) {
                            let src_pos = (y * columns + x) * bytes_per_pixel;
                            if src_pos + bytes_per_pixel <= frame_size {
                                new_bytes.extend_from_slice(&frame_bytes[src_pos..src_pos+bytes_per_pixel]);
                            }
                        }
                    }
                }
                
                // 更新图像尺寸和位深度相关标签
                output_dataset.insert(
                    dictionary::ROWS.tag,
                    DataElementValue::new_binary(
                        ValueRepresentation::UnsignedShort,
                        Rc::new((new_rows as u16).to_le_bytes().to_vec())
                    ).unwrap()
                );
                output_dataset.insert(
                    dictionary::COLUMNS.tag,
                    DataElementValue::new_binary(
                        ValueRepresentation::UnsignedShort,
                        Rc::new((new_columns as u16).to_le_bytes().to_vec())
                    ).unwrap()
                );
                output_dataset.insert(
                    dictionary::BITS_ALLOCATED.tag,
                    DataElementValue::new_binary(
                        ValueRepresentation::UnsignedShort,
                        Rc::new((bits_allocated as u16).to_le_bytes().to_vec())
                    ).unwrap()
                );
                output_dataset.insert(
                    dictionary::BITS_STORED.tag,
                    DataElementValue::new_binary(
                        ValueRepresentation::UnsignedShort,
                        Rc::new((bits_stored as u16).to_le_bytes().to_vec())
                    ).unwrap()
                );
                output_dataset.insert(
                    dictionary::HIGH_BIT.tag,
                    DataElementValue::new_binary(
                        ValueRepresentation::UnsignedShort,
                        Rc::new((high_bit as u16).to_le_bytes().to_vec())
                    ).unwrap()
                );
                
                // 添加帧数信息
                if number_of_frames > 1 {
                    let frame_str = format!("{}\0", number_of_frames);
                    output_dataset.insert(
                        dictionary::NUMBER_OF_FRAMES.tag,
                        DataElementValue::new_binary(
                            ValueRepresentation::IntegerString,
                            Rc::new(frame_str.into_bytes())
                        ).unwrap()
                    );
                }
                
                // 创建新的像素数据值
                output_dataset.insert(
                    dictionary::PIXEL_DATA.tag,
                    DataElementValue::new_binary(
                        ValueRepresentation::OtherWordString,
                        Rc::new(new_bytes)
                    ).unwrap()
                );
            } else {
                // 不降采样，直接使用原始数据
                output_dataset.insert(
                    dictionary::PIXEL_DATA.tag,
                    DataElementValue::new_binary(
                        ValueRepresentation::OtherWordString,
                        Rc::new(bytes.to_vec())
                    ).unwrap()
                );
            }
        }
    }

    // 6. 写入新的数据集到内存缓冲区
    let mut output_data = Vec::new();
    let mut bytes_callback = |bytes: Rc<Vec<u8>>| {
        output_data.extend_from_slice(&bytes);
        Ok(())
    };
    
    data_set_to_bytes(&output_dataset, &mut bytes_callback, &P10WriteConfig::default())?;

    Ok(output_data)
}


#[wasm_bindgen]
pub fn export_compressed_dicom(file_data: &[u8], quality: u32) -> Result<Vec<u8>, JsValue> {
    web_sys::console::log_1(&format!("开始压缩 DICOM 文件，质量参数: {}", quality).into());

    if file_data.len() < 132 {
        return Err("文件太小，不是有效的DICOM文件".into());
    }

    let preamble = &file_data[128..132];
    if preamble != b"DICM" {
        return Err("不是有效的DICOM文件格式".into());
    }

    // 调用流式压缩函数
    match compress_dicom_stream(file_data, quality) {
        Ok(output_data) => {
            web_sys::console::log_1(&format!("压缩完成，输出大小: {} 字节", output_data.len()).into());
            Ok(output_data)
        },
        Err(e) => {
            web_sys::console::error_1(&format!("压缩失败: {}", e).into());
            Err(format!("压缩失败: {}", e).into())
        }
    }
} 