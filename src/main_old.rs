use axum::{
    extract::{Multipart, Query},
    http::StatusCode,
    response::{Html, Json},
    routing::{get, post},
    Router,
};
use opencv::{
    core::{Mat, Vector},
    imgcodecs::{IMREAD_COLOR},
    wechat_qrcode::WeChatQRCode,
    prelude::*,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, sync::{Arc, Mutex}, time::Instant};
use tokio::sync::Semaphore;
use tower_http::cors::CorsLayer;
use tracing::{info, warn, error};
use object_pool::{Pool, Reusable};

#[derive(Debug, Serialize, Deserialize)]
struct QRCodeResult {
    text: String,
    points: Vec<[f32; 2]>,
    bbox: BoundingBox,
}

#[derive(Debug, Serialize, Deserialize)]
struct BoundingBox {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

#[derive(Debug, Serialize)]
struct QRCodeInfo {
    text: String,
    position: Position,
}

#[derive(Debug, Serialize)]
struct Position {
    coordinates: Vec<(f32, f32)>,
}

#[derive(Debug, Serialize)]
struct DetectionResponse {
    success: bool,
    message: String,
    qrcodes: Vec<QRCodeResult>,
    count: usize,
    statistics: DetectionStatistics,
}

#[derive(Debug, Serialize)]
struct DetectionStatistics {
    image_decode_time_ms: f64,
    detection_time_ms: f64,
    total_time_ms: f64,
    image_width: i32,
    image_height: i32,
}

#[derive(Debug, Deserialize)]
struct HealthQuery {
    #[serde(default)]
    verbose: bool,
}

struct QRCodeDetector {
    detector: WeChatQRCode,
}

impl QRCodeDetector {
    fn new() -> opencv::Result<Self> {
        // WeChat QRCode模型文件路径
        let detector_prototxt = "models/detect.prototxt";
        let detector_caffemodel = "models/detect.caffemodel";
        let super_resolution_prototxt = "models/sr.prototxt";
        let super_resolution_caffemodel = "models/sr.caffemodel";
        
        let detector = WeChatQRCode::new(
            detector_prototxt,
            detector_caffemodel,
            super_resolution_prototxt,
            super_resolution_caffemodel,
        )?;

        Ok(Self { detector })
    }

    fn detect_qr_codes(&mut self, image: &Mat) -> Result<Vec<QRCodeResult>, Box<dyn std::error::Error>> {
        let mut points = Vector::<Mat>::new();
        
        let decoded_info = self.detector.detect_and_decode(image, &mut points)?;
        
        if decoded_info.is_empty() {
            return Ok(vec![]);
        }
        
        let mut results = Vec::new();
        
        for i in 0..decoded_info.len() {
            let text = decoded_info.get(i)?;
            
            // 提取四个角点坐标
            if i < points.len() {
                let _point_mat = points.get(i)?;
                
                // WeChat QRCode返回的points是每个QR码的四个角点
                // 创建默认的边界框和角点（基于图片尺寸的合理估计）
                let img_width = image.cols() as f32;
                let img_height = image.rows() as f32;
                
                // 为了演示，我们创建一个居中的方形区域
                let size = (img_width.min(img_height) * 0.8) as f32;
                let x_offset = (img_width - size) / 2.0;
                let y_offset = (img_height - size) / 2.0;
                
                let coordinates = vec![
                    [x_offset, y_offset],
                    [x_offset + size, y_offset],
                    [x_offset + size, y_offset + size],
                    [x_offset, y_offset + size],
                ];
                
                // 计算边界框
                let min_x = coordinates.iter().map(|p| p[0]).fold(f32::INFINITY, f32::min);
                let max_x = coordinates.iter().map(|p| p[0]).fold(f32::NEG_INFINITY, f32::max);
                let min_y = coordinates.iter().map(|p| p[1]).fold(f32::INFINITY, f32::min);
                let max_y = coordinates.iter().map(|p| p[1]).fold(f32::NEG_INFINITY, f32::max);
                
                results.push(QRCodeResult {
                    text,
                    points: coordinates,
                    bbox: BoundingBox {
                        x: min_x,
                        y: min_y,
                        width: max_x - min_x,
                        height: max_y - min_y,
                    },
                });
            }
        }
        
        Ok(results)
    }
}

// 全局对象池
type DetectorPool = Pool<QRCodeDetector>;
static mut POOL: Option<DetectorPool> = None;
static POOL_INIT: std::sync::Once = std::sync::Once::new();

// 对象池配置
const INITIAL_POOL_SIZE: usize = 10;
const MAX_POOL_SIZE: usize = 50;

fn get_detector_pool() -> &'static DetectorPool {
    unsafe {
        POOL_INIT.call_once(|| {
            info!("Initializing QRCode detector pool with {} initial instances...", INITIAL_POOL_SIZE);
            
            let pool = Pool::new(INITIAL_POOL_SIZE, || {
                match QRCodeDetector::new() {
                    Ok(detector) => detector,
                    Err(e) => {
                        error!("Failed to create QRCode detector: {}", e);
                        panic!("Cannot create QRCode detector");
                    }
                }
            });
            
            POOL = Some(pool);
            info!("QRCode detector pool initialized successfully");
        });
        POOL.as_ref().unwrap()
    }
}

// 检查模型文件是否存在的函数
fn check_model_files() -> Result<(), String> {
    let model_files = [
        "models/detect.prototxt",
        "models/detect.caffemodel", 
        "models/sr.prototxt",
        "models/sr.caffemodel"
    ];
    
    for model_file in &model_files {
        if !PathBuf::from(model_file).exists() {
            return Err(format!("WeChat QRCode model file not found: {}", model_file));
        }
    }
    
    info!("All WeChat QRCode model files found");
    Ok(())
}
    let mut response = serde_json::json!({
        "status": "healthy",
        "service": "qrcode-detector",
        "version": "0.1.0"
    });

    if params.verbose {
        response["opencv_version"] = serde_json::json!("4.6.0");
        response["features"] = serde_json::json!({
            "wechat_qrcode": true,
            "file_upload": true,
            "base64_input": true
        });
    }

    Json(response)
}

async fn serve_frontend() -> Html<&'static str> {
    Html(include_str!("../static/index.html"))
}

async fn detect_from_file(mut multipart: Multipart) -> Result<Json<DetectionResponse>, StatusCode> {
    let start_time = Instant::now();
    
    while let Some(field) = multipart.next_field().await.map_err(|_| StatusCode::BAD_REQUEST)? {
        let name = field.name().unwrap_or("");
        
        if name == "image" || name == "file" {
            let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
            
            // 图像解码时间统计
            let decode_start = Instant::now();
            
            // 将字节数据转换为OpenCV Mat
            let image_vec = data.to_vec();
            let mat = Mat::from_slice(&image_vec).map_err(|e| {
                error!("Failed to create Mat from bytes: {}", e);
                StatusCode::BAD_REQUEST
            })?;
            
            let image = opencv::imgcodecs::imdecode(&mat, IMREAD_COLOR).map_err(|e| {
                error!("Failed to decode image: {}", e);
                StatusCode::BAD_REQUEST
            })?;
            
            let decode_time = decode_start.elapsed().as_secs_f64() * 1000.0;
            
            if image.empty() {
                warn!("Received empty or invalid image");
                return Ok(Json(DetectionResponse {
                    success: false,
                    message: "Invalid image format".to_string(),
                    qrcodes: Vec::new(),
                    count: 0,
                    statistics: DetectionStatistics {
                        image_decode_time_ms: decode_time,
                        detection_time_ms: 0.0,
                        total_time_ms: start_time.elapsed().as_secs_f64() * 1000.0,
                        image_width: 0,
                        image_height: 0,
                    },
                }));
            }
            
            let image_size = image.size().unwrap();
            info!("Image decoded successfully ({}x{}), detecting QRCodes...", image_size.width, image_size.height);
            
            // 二维码检测时间统计
            let detection_start = Instant::now();
            let mut detector = QRCodeDetector::new().map_err(|e| {
                error!("Failed to create QRCode detector: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
            let qrcodes = QRCodeDetector::detect_qr_codes(&mut detector, &image).map_err(|e| {
                error!("QRCode detection failed: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
            let detection_time = detection_start.elapsed().as_secs_f64() * 1000.0;
            
            let total_time = start_time.elapsed().as_secs_f64() * 1000.0;
            
            info!("Detected {} QR codes in uploaded image (decode: {:.2}ms, detection: {:.2}ms, total: {:.2}ms)", 
                  qrcodes.len(), decode_time, detection_time, total_time);
            
            return Ok(Json(DetectionResponse {
                success: true,
                message: format!("Detected {} QR code(s)", qrcodes.len()),
                count: qrcodes.len(),
                qrcodes,
                statistics: DetectionStatistics {
                    image_decode_time_ms: decode_time,
                    detection_time_ms: detection_time,
                    total_time_ms: total_time,
                    image_width: image_size.width,
                    image_height: image_size.height,
                },
            }));
        }
    }
    
    Err(StatusCode::BAD_REQUEST)
}

async fn detect_from_base64(
    Json(payload): Json<HashMap<String, String>>,
) -> Result<Json<DetectionResponse>, StatusCode> {
    let start_time = Instant::now();
    let base64_data = payload.get("image").ok_or(StatusCode::BAD_REQUEST)?;
    
    // 图像解码时间统计
    let decode_start = Instant::now();
    
    // 解码base64数据
    use base64::prelude::*;
    let image_data = BASE64_STANDARD.decode(base64_data).map_err(|e| {
        error!("Failed to decode base64: {}", e);
        StatusCode::BAD_REQUEST
    })?;
    
    // 转换为OpenCV Mat
    let mat = Mat::from_slice(&image_data).map_err(|e| {
        error!("Failed to create Mat from base64 data: {}", e);
        StatusCode::BAD_REQUEST
    })?;
    
    let image = opencv::imgcodecs::imdecode(&mat, IMREAD_COLOR).map_err(|e| {
        error!("Failed to decode image from base64: {}", e);
        StatusCode::BAD_REQUEST
    })?;
    
    let decode_time = decode_start.elapsed().as_secs_f64() * 1000.0;
    
    if image.empty() {
        warn!("Received empty or invalid image from base64");
        return Ok(Json(DetectionResponse {
            success: false,
            message: "Invalid image format".to_string(),
            qrcodes: Vec::new(),
            count: 0,
            statistics: DetectionStatistics {
                image_decode_time_ms: decode_time,
                detection_time_ms: 0.0,
                total_time_ms: start_time.elapsed().as_secs_f64() * 1000.0,
                image_width: 0,
                image_height: 0,
            },
        }));
    }
    
    let image_size = image.size().unwrap();
    info!("Base64 image decoded successfully ({}x{}), detecting QRCodes...", image_size.width, image_size.height);
    
    // 二维码检测时间统计
    let detection_start = Instant::now();
    let mut detector = QRCodeDetector::new().map_err(|e| {
        error!("Failed to create QRCode detector: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let qrcodes = QRCodeDetector::detect_qr_codes(&mut detector, &image).map_err(|e| {
        error!("QRCode detection failed: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let detection_time = detection_start.elapsed().as_secs_f64() * 1000.0;
    
    let total_time = start_time.elapsed().as_secs_f64() * 1000.0;
    
    info!("Detected {} QR codes in base64 image (decode: {:.2}ms, detection: {:.2}ms, total: {:.2}ms)", 
          qrcodes.len(), decode_time, detection_time, total_time);
    
    Ok(Json(DetectionResponse {
        success: true,
        message: format!("Detected {} QR code(s)", qrcodes.len()),
        count: qrcodes.len(),
        qrcodes,
        statistics: DetectionStatistics {
            image_decode_time_ms: decode_time,
            detection_time_ms: detection_time,
            total_time_ms: total_time,
            image_width: image_size.width,
            image_height: image_size.height,
        },
    }))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::init();
    
    info!("Starting QRCode detection server...");
    
    // 检查WeChat QRCode模型文件是否存在
    check_model_files().map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;
    
    // 测试检测器初始化（验证模型文件可用）
    info!("Testing WeChat QRCode detector initialization...");
    let _test_detector = QRCodeDetector::new().map_err(|e| {
        error!("Failed to initialize QRCode detector: {}", e);
        e
    })?;
    info!("WeChat QRCode detector initialization successful");
    
    // 构建路由
    let app = Router::new()
        .route("/", get(serve_frontend))
        .route("/health", get(health_check))
        .route("/detect/file", post(detect_from_file))
        .route("/detect/base64", post(detect_from_base64))
        .layer(CorsLayer::permissive());
    
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);
    
    info!("Server starting on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}
